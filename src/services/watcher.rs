use std::sync::Arc;

use sea_orm::DatabaseConnection;
use tokio::sync::RwLock;

use crate::state::ScanState;

/// Start watching configured libraries for file changes.
///
/// This is a stub — full implementation would use the `notify` crate to set up
/// filesystem watchers on each library path and react to create / modify /
/// delete events by updating the database and/or triggering a scan.
pub async fn start_watching(
    _db: DatabaseConnection,
    _scan_state: Arc<RwLock<ScanState>>,
    _library_paths: Vec<(i32, String)>,
) {
    tracing::info!(
        "File watcher would watch {} paths (stub)",
        _library_paths.len()
    );
    // Full implementation:
    // 1. Create notify::RecommendedWatcher
    // 2. Watch each library path recursively
    // 3. On file events, update database accordingly
    // 4. Respect shutdown signal
}
