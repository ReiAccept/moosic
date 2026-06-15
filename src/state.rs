use redis::aio::MultiplexedConnection;
use sea_orm::DatabaseConnection;

/// Shared application state, accessible from handlers via
/// `axum::extract::State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub redis: MultiplexedConnection,
}
