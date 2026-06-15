mod config;
mod db;
mod entities;
mod handlers;
mod redis;
mod router;
mod state;

use config::Config;
use state::AppState;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // load configuration from JSON file
    let config = Config::load();

    // connect to database and run pending migrations
    let db = db::connect(&config.database).await;

    // connect to Redis
    let redis = redis::connect(&config.redis).await;
    tracing::info!("Redis connected: url={}", config.redis.url);

    // build our application with shared state
    let state = AppState { db, redis };
    let app = router::create_router(state);

    // bind to the configured address and port
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {addr}: {e}"));

    tracing::info!("Listening on {addr}");

    let _ = axum::serve(listener, app).await;
}
