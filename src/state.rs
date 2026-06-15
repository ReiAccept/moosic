use sea_orm::DatabaseConnection;

use crate::cache::CacheBackend;

/// Shared application state, accessible from handlers via
/// `axum::extract::State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub cache: CacheBackend,
    /// Database backend identifier, e.g. `"sqlite"`.
    pub db_backend: &'static str,
    /// Server listen address.
    pub server_host: String,
    /// Server listen port.
    pub server_port: u16,
}
