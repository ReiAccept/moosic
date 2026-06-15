mod config;
mod entities;
mod handlers;
mod router;

use config::Config;
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // load configuration from JSON file
    let config = Config::load();
    tracing::info!("Loaded config: database.url={}", config.database.url);

    // connect to SQLite database
    let db = Database::connect(&config.database.url)
        .await
        .expect("Failed to connect to database");

    // run pending migrations
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Database connected and migrations applied");

    // build our application with a shared database connection
    let app = router::create_router(db);

    // bind to the configured address and port
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {addr}: {e}"));

    tracing::info!("Listening on {addr}");

    let _ = axum::serve(listener, app).await;
}
