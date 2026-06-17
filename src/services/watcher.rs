use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use notify::{Event, EventKind, RecursiveMode, Watcher as _};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tokio::sync::{oneshot, RwLock};

use crate::entities::{albums, artists, libraries, songs};
use crate::error::AppError;
use crate::services::{metadata, scanner};
use crate::state::ScanState;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Handle to the running watcher background task.
///
/// Dropping this handle does **not** stop the watcher; call [`WatcherHandle::shutdown`]
/// to signal a graceful stop.
pub struct WatcherHandle {
    shutdown_tx: oneshot::Sender<()>,
}

impl WatcherHandle {
    /// Signal the watcher to stop gracefully.
    pub fn shutdown(self) {
        let _ = self.shutdown_tx.send(());
    }
}

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

/// A pending filesystem event collected during the debounce window.
struct PendingEvent {
    path: PathBuf,
    library_id: i32,
    /// Whether the file currently exists on disk at the end of the window.
    exists: bool,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Start watching all libraries that have `watch_enabled = 1`.
///
/// Spawns a background task that listens for filesystem events and incrementally
/// updates the database.  Returns a [`WatcherHandle`] that can be used to signal
/// a graceful shutdown.
pub async fn start_watching(
    db: DatabaseConnection,
    scan_state: Arc<RwLock<ScanState>>,
) -> WatcherHandle {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    tokio::spawn(async move {
        if let Err(e) = run_watcher(db, scan_state, shutdown_rx).await {
            tracing::error!("Watcher exited with error: {e}");
        }
        tracing::info!("Watcher stopped");
    });

    WatcherHandle { shutdown_tx }
}

// ---------------------------------------------------------------------------
// Main watcher loop
// ---------------------------------------------------------------------------

async fn run_watcher(
    db: DatabaseConnection,
    scan_state: Arc<RwLock<ScanState>>,
    shutdown_rx: oneshot::Receiver<()>,
) -> Result<(), AppError> {
    // Load libraries that have watching enabled
    let libs = libraries::Entity::find()
        .filter(libraries::Column::WatchEnabled.eq(1))
        .all(&db)
        .await?;

    if libs.is_empty() {
        tracing::info!("Watcher: no libraries with watch_enabled=1, exiting");
        return Ok(());
    }

    tracing::info!(
        "Watcher: starting file monitoring on {} librar{}",
        libs.len(),
        if libs.len() == 1 { "y" } else { "ies" }
    );

    // Build a channel bridge: notify callback (sync) → tokio mpsc (async)
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(PathBuf, i32)>();

    // Create the OS filesystem watcher
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        match res {
            Ok(event) => {
                // We only care about file-level create/modify/remove events
                if !is_relevant_event(&event) {
                    return;
                }
                for path in &event.paths {
                    // library_id is embedded in the path — we resolve it later
                    // For now, send 0 as placeholder; the debounce step will
                    // re-resolve the library from the path → library map.
                    let _ = tx.send((path.clone(), 0));
                }
            }
            Err(e) => {
                tracing::error!("Watcher: notify error: {e}");
            }
        }
    })
    .map_err(|e| AppError::internal(format!("Failed to create filesystem watcher: {e}")))?;

    // Build a path-prefix → library_id map for routing events to the correct library
    let path_map: Vec<(PathBuf, i32)> = libs
        .iter()
        .map(|lib| (PathBuf::from(&lib.path), lib.id))
        .collect();

    // Watch each library path recursively
    for (path, lib_id) in &path_map {
        if !path.exists() {
            tracing::warn!(
                "Watcher: library {lib_id} path does not exist: {}",
                path.display()
            );
            continue;
        }
        watcher
            .watch(path.as_path(), RecursiveMode::Recursive)
            .map_err(|e| {
                AppError::internal(format!(
                    "Failed to watch {}: {e}",
                    path.display()
                ))
            })?;
        tracing::info!("Watcher: watching {}", path.display());
    }

    // Store the path map for event routing
    let path_map = Arc::new(path_map);

    // ---------- Event loop ----------
    tokio::pin!(shutdown_rx);

    loop {
        // Wait for the first event (or shutdown)
        let first = tokio::select! {
            _ = &mut shutdown_rx => {
                tracing::info!("Watcher: received shutdown signal");
                break;
            }
            ev = rx.recv() => {
                match ev {
                    Some(e) => e,
                    None => break, // channel closed
                }
            }
        };

        // Collect events into a batch with a 2-second debounce window
        let mut pending: HashMap<PathBuf, bool> = HashMap::new();
        pending.insert(first.0.clone(), first.0.exists());

        let deadline = tokio::time::sleep(Duration::from_secs(2));
        tokio::pin!(deadline);

        // Drain any additional events that arrive within the debounce window
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => break,
                _ = &mut deadline => break,
                ev = rx.recv() => {
                    match ev {
                        Some((path, _lib_id)) => {
                            // Record whether the file still exists (latest event wins)
                            // We check existence at processing time, so just dedup the path
                            pending.entry(path).or_insert(true);
                            // Reset deadline — keep collecting while events are arriving
                            deadline.as_mut().reset(tokio::time::Instant::now() + Duration::from_secs(2));
                        }
                        None => break,
                    }
                }
            }
        }

        // Resolve library_id for each pending path and check existence
        let events = resolve_events(&pending, &path_map);

        // If there's an active scan running for any affected library, skip those events
        let skip_libs = active_scan_libraries(&scan_state).await;

        let mut processed = 0usize;
        for ev in &events {
            if skip_libs.contains(&ev.library_id) {
                tracing::debug!(
                    "Watcher: skipping {} (library {} is being scanned)",
                    ev.path.display(),
                    ev.library_id
                );
                continue;
            }

            if let Err(e) = process_event(&db, ev).await {
                tracing::warn!("Watcher: failed to process {}: {e}", ev.path.display());
            } else {
                processed += 1;
            }
        }

        if processed > 0 {
            tracing::debug!("Watcher: processed {processed} events");
        }
    }

    // Cleanup
    drop(watcher);
    Ok(())
}

// ---------------------------------------------------------------------------
// Event helpers
// ---------------------------------------------------------------------------

/// Check whether a notify event is one we care about (file create/modify/remove).
fn is_relevant_event(event: &Event) -> bool {
    matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    )
}

/// Resolve pending paths to `PendingEvent` structs with library_id and existence.
fn resolve_events(
    pending: &HashMap<PathBuf, bool>,
    path_map: &[(PathBuf, i32)],
) -> Vec<PendingEvent> {
    let mut events = Vec::with_capacity(pending.len());

    for (path, _) in pending {
        // Find which library this path belongs to (longest prefix match)
        let lib_id = path_map
            .iter()
            .filter(|(prefix, _)| path.starts_with(prefix))
            .max_by_key(|(prefix, _)| prefix.as_os_str().len())
            .map(|(_, id)| *id);

        let lib_id = match lib_id {
            Some(id) => id,
            None => {
                tracing::debug!(
                    "Watcher: ignoring event for path outside known libraries: {}",
                    path.display()
                );
                continue;
            }
        };

        // Only process audio files
        if path.exists() && !metadata::is_audio_file(path) {
            continue;
        }

        // For removed files we can't check is_audio_file, so we check
        // whether it has a known audio extension; if not, skip
        if !path.exists() && !metadata::is_audio_file(path) {
            continue;
        }

        events.push(PendingEvent {
            path: path.clone(),
            library_id: lib_id,
            exists: path.exists(),
        });
    }

    events
}

/// Return the set of library IDs currently being scanned.
async fn active_scan_libraries(scan_state: &Arc<RwLock<ScanState>>) -> HashSet<i32> {
    let state = scan_state.read().await;
    match &state.active {
        Some(progress) if progress.status == crate::state::ScanStatus::Scanning => {
            progress.library_ids.iter().copied().collect()
        }
        _ => HashSet::new(),
    }
}

// ---------------------------------------------------------------------------
// Event processing
// ---------------------------------------------------------------------------

/// Process a single pending event.
async fn process_event(db: &DatabaseConnection, ev: &PendingEvent) -> Result<(), AppError> {
    if ev.exists {
        // File was created or modified — scan it
        let mtime_ms = std::fs::metadata(&ev.path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        let size_bytes = std::fs::metadata(&ev.path)
            .ok()
            .map(|m| m.len() as i64)
            .unwrap_or(0);

        let entry = scanner::WalkEntry {
            file_path: ev.path.to_string_lossy().to_string(),
            mtime_ms,
            size_bytes,
        };

        tracing::debug!("Watcher: scanning {}", entry.file_path);
        scanner::scan_single_file(db, ev.library_id, &entry).await
    } else {
        // File was deleted — remove from database
        tracing::debug!("Watcher: removing {}", ev.path.display());
        delete_song_by_path(db, ev.library_id, &ev.path).await
    }
}

// ---------------------------------------------------------------------------
// Delete handling
// ---------------------------------------------------------------------------

/// Delete a song by its file path and clean up orphan artists/albums in the
/// same library.
async fn delete_song_by_path(
    db: &DatabaseConnection,
    library_id: i32,
    path: &std::path::Path,
) -> Result<(), AppError> {
    let file_path = path.to_string_lossy().to_string();

    // Find the song
    let song = songs::Entity::find()
        .filter(songs::Column::FilePath.eq(&file_path))
        .one(db)
        .await?;

    let song = match song {
        Some(s) => s,
        None => {
            tracing::debug!("Watcher: song not found in DB for deleted file: {file_path}");
            return Ok(());
        }
    };

    let artist_id = song.artist_id;
    let album_id = song.album_id;

    // Delete the song (cascades to scrobbles, bookmarks, playlist_songs, lyrics)
    songs::Entity::delete_by_id(song.id).exec(db).await?;

    // Clean up orphan artist
    let artist_still_has_songs = songs::Entity::find()
        .filter(songs::Column::ArtistId.eq(artist_id))
        .filter(songs::Column::LibraryId.eq(library_id))
        .one(db)
        .await?
        .is_some();

    if !artist_still_has_songs {
        artists::Entity::delete_by_id(artist_id).exec(db).await?;
        tracing::debug!("Watcher: removed orphan artist id={artist_id}");
    }

    // Clean up orphan album
    if let Some(aid) = album_id {
        let album_still_has_songs = songs::Entity::find()
            .filter(songs::Column::AlbumId.eq(aid))
            .filter(songs::Column::LibraryId.eq(library_id))
            .one(db)
            .await?
            .is_some();

        if !album_still_has_songs {
            albums::Entity::delete_by_id(aid).exec(db).await?;
            tracing::debug!("Watcher: removed orphan album id={aid}");
        }
    }

    tracing::info!("Watcher: deleted song id={} ({})", song.id, file_path);
    Ok(())
}
