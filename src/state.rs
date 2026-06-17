use std::sync::Arc;
use std::time::Instant;

use sea_orm::DatabaseConnection;
use tokio::sync::RwLock;

use crate::cache::CacheBackend;

/// Shared application state, accessible from handlers via
/// `axum::extract::State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub cache: CacheBackend,
    /// Server listen address.
    pub server_host: String,
    /// Server listen port.
    pub server_port: u16,
    /// Server start time, used for uptime calculation.
    pub start_time: Instant,
    /// Current scan task state, shared between API and background task.
    pub scan_state: Arc<RwLock<ScanState>>,
}

/// State of the current (or most recent) scan task.
#[derive(Clone, Debug, Default)]
pub struct ScanState {
    pub active: Option<ScanProgress>,
}

/// Progress of a running or completed scan.
#[derive(Clone, Debug)]
pub struct ScanProgress {
    pub scan_id: String,
    pub library_ids: Vec<i32>,
    pub status: ScanStatus,
    pub files_scanned: i64,
    pub files_total: i64,
    pub started_at: i64,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ScanStatus {
    Scanning,
    Completed,
    Failed,
    Cancelled,
}
