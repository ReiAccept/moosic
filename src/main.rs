mod cache;
mod config;
mod db;
mod entities;
mod handlers;
mod router;
mod state;

use cache::{CacheBackend, MemoryCache, RedisCache};
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

    // build our application with shared state
    let state = AppState {
        db,
        cache,
        db_backend,
        server_host: config.server.host.clone(),
        server_port: config.server.port,
    };
    let app = router::create_router(state);

    // bind to the configured address and port
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {addr}: {e}"));

    tracing::info!("Listening on {addr}");

    let _ = axum::serve(listener, app).await;
}
