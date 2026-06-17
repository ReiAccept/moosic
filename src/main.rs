mod cache;
mod config;
mod db;
mod entities;
mod error;
mod handlers;
mod middleware;
mod router;
mod services;
mod state;
mod utils;

use cache::init as init_cache;
use config::Config;
use state::AppState;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let config = Config::load();

    // Parse log level from config
    let level: tracing::Level = config.log.level.parse().unwrap_or_else(|_| {
        eprintln!(
            "Invalid log level '{}', falling back to INFO",
            config.log.level
        );
        tracing::Level::INFO
    });

    // Ensure log directory exists
    let log_path = std::path::Path::new(&config.log.path);
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)
            .unwrap_or_else(|e| panic!("Failed to create log directory {:?}: {e}", parent));
    }

    // File appender (no rotation, writes to the configured path)
    let file_appender = tracing_appender::rolling::never(
        log_path.parent().unwrap_or(std::path::Path::new(".")),
        log_path
            .file_name()
            .unwrap_or(std::ffi::OsStr::new("moosic.log")),
    );
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Build subscriber: stdout layer + file layer, both at the configured level
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::Layer;
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_filter(tracing_subscriber::filter::LevelFilter::from_level(
                    level,
                )),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_filter(tracing_subscriber::filter::LevelFilter::from_level(
                    level,
                )),
        )
        .init();

    let db = db::connect(&config.database).await;

    seed_admin_user(&db).await;

    let cache = init_cache(&config.cache).await;

    // Spawn periodic cleanup task for expired sessions / shares
    let cleanup_db = db.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);
            use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
            use crate::entities::{sessions, shares};
            let _ = sessions::Entity::delete_many()
                .filter(sessions::Column::ExpiresAt.lt(now))
                .exec(&cleanup_db)
                .await;
            let _ = shares::Entity::delete_many()
                .filter(shares::Column::ExpiresAt.lt(now))
                .filter(shares::Column::ExpiresAt.is_not_null())
                .exec(&cleanup_db)
                .await;
            tracing::debug!("Cleanup task ran");
        }
    });

    let scan_state = Arc::new(RwLock::new(state::ScanState::default()));

    // Start the filesystem watcher for libraries with watch_enabled
    let watcher_handle = services::watcher::start_watching(db.clone(), scan_state.clone()).await;

    let state = AppState {
        db,
        cache,
        config: config.clone(),
        scan_state,
    };
    let app = router::create_router(state);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {addr}: {e}"));

    tracing::info!("Listening on {addr}");

    let _ = axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
            tracing::info!("Shutting down...");
            watcher_handle.shutdown();
        })
        .await;
}

/// Create a default admin user (admin / 123456) if no users exist.
async fn seed_admin_user(db: &sea_orm::DatabaseConnection) {
    use argon2::password_hash::{PasswordHasher, SaltString};
    use rand_core::OsRng;
    use argon2::Argon2;
    use sea_orm::{ActiveModelTrait, ActiveValue::*, EntityTrait};

    use crate::entities::users;

    // Check if any user already exists
    let existing = users::Entity::find().one(db).await.unwrap_or(None);
    if existing.is_some() {
        return;
    }

    // Hash the default password
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(b"123456", &salt)
        .expect("Failed to hash default password")
        .to_string();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let admin = users::ActiveModel {
        username: Set("admin".into()),
        password_hash: Set(hash),
        email: Set(None),
        privs: Set(r#"{"edit_user":true,"edit_library":true,"read_server":true}"#.into()),
        scrobbling_enabled: Set(1),
        max_bit_rate: Set(0),
        is_enabled: Set(1),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    admin.insert(db).await.expect("Failed to create default admin user");
    tracing::info!("Default admin user created (admin / 123456)");
}
