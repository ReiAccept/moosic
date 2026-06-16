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

use cache::{CacheBackend, MemoryCache, RedisCache};
use config::Config;
use state::AppState;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // load configuration from JSON file
    let config = Config::load();

    // connect to database and run pending migrations
    let db = db::connect(&config.database).await;

    // Seed default admin user if none exists
    seed_admin_user(&db).await;

    // select cache backend based on configuration
    let cache = if config.redis.enabled {
        let redis_cache = RedisCache::connect(&config.redis).await;
        tracing::info!("Redis connected (cache): url={}", config.redis.url);
        CacheBackend::Redis(redis_cache)
    } else {
        tracing::info!("Using in-memory cache (DashMap)");
        CacheBackend::Memory(MemoryCache::new())
    };

    // determine database backend name for status reporting
    let db_backend = match &config.database {
        config::Database::Sqlite { .. } => "sqlite",
    };

    // Spawn periodic cleanup task for expired sessions / shares / password resets
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
            use crate::entities::{sessions, shares, password_resets};
            let _ = sessions::Entity::delete_many()
                .filter(sessions::Column::ExpiresAt.lt(now))
                .exec(&cleanup_db)
                .await;
            let _ = shares::Entity::delete_many()
                .filter(shares::Column::ExpiresAt.lt(now))
                .filter(shares::Column::ExpiresAt.is_not_null())
                .exec(&cleanup_db)
                .await;
            let _ = password_resets::Entity::delete_many()
                .filter(password_resets::Column::ExpiresAt.lt(now))
                .exec(&cleanup_db)
                .await;
            tracing::debug!("Cleanup task ran");
        }
    });

    // build our application with shared state
    let state = AppState {
        db,
        cache,
        db_backend,
        server_host: config.server.host.clone(),
        server_port: config.server.port,
        start_time: std::time::Instant::now(),
        scan_state: Arc::new(RwLock::new(state::ScanState::default())),
    };
    let app = router::create_router(state);

    // bind to the configured address and port
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {addr}: {e}"));

    tracing::info!("Listening on {addr}");

    // Graceful shutdown on Ctrl+C
    let _ = axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
            tracing::info!("Shutting down...");
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
