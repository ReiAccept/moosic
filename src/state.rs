use std::sync::Arc;

use sea_orm::DatabaseConnection;
use tokio::sync::RwLock;

use crate::cache::CacheBackend;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub cache: CacheBackend,
    pub config: Config,
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
