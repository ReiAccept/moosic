use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use rand::Rng;
use sea_orm::{ActiveModelTrait, ActiveValue::*, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tokio::sync::RwLock;

use crate::entities::{albums, artists, cover_art, libraries, songs};
use crate::error::AppError;
use crate::services::metadata;
use crate::state::{ScanProgress, ScanState, ScanStatus};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Generate a unique scan identifier.
pub fn generate_scan_id() -> String {
    let token: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();
    format!("scan_{}", token.to_lowercase())
}

/// Start a scan for the given library IDs as a background task.
///
/// Returns the new `scan_id`.  If `library_ids` is empty, all libraries are
/// scanned.  Returns a conflict error when another scan is already running.
pub async fn start_scan(
    db: DatabaseConnection,
    scan_state: Arc<RwLock<ScanState>>,
    library_ids: Vec<i32>,
) -> Result<String, AppError> {
    let mut state = scan_state.write().await;
    if state.active.is_some() {
        return Err(AppError::conflict("A scan is already in progress"));
    }

    let scan_id = generate_scan_id();

    state.active = Some(ScanProgress {
        scan_id: scan_id.clone(),
        library_ids: library_ids.clone(),
        status: ScanStatus::Scanning,
        files_scanned: 0,
        files_total: 0,
        started_at: now_ms(),
        error: None,
    });
    drop(state);

    // Spawn the actual scanning work
    let db2 = db.clone();
    let state2 = scan_state.clone();
    let sid = scan_id.clone();
    tokio::spawn(async move {
        if let Err(e) = run_scan(&db2, &state2, &sid, &library_ids).await {
            tracing::error!("Scan {sid} failed: {e}");
            let mut s = state2.write().await;
            if let Some(ref mut p) = s.active {
                if p.scan_id == sid {
                    p.status = ScanStatus::Failed;
                    p.error = Some(e.to_string());
                }
            }
        }
    });

    Ok(scan_id)
}

/// Cancel an active scan by its scan ID.
pub async fn cancel_scan(
    scan_state: &Arc<RwLock<ScanState>>,
    scan_id: &str,
) -> Result<(), AppError> {
    let mut state = scan_state.write().await;
    match state.active.as_mut() {
        Some(progress) if progress.scan_id == scan_id => {
            progress.status = ScanStatus::Cancelled;
            Ok(())
        }
        _ => Err(AppError::not_found("Scan not found or not running")),
    }
}

/// Return a snapshot of the current scan progress, if any.
pub async fn get_status(scan_state: &Arc<RwLock<ScanState>>) -> Option<ScanProgress> {
    scan_state.read().await.active.clone()
}

/// Check whether this scan has been cancelled.
fn is_cancelled(state: &Arc<RwLock<ScanState>>, scan_id: &str) -> bool {
    // try_read is non-blocking — fine for a periodic check
    if let Ok(s) = state.try_read() {
        if let Some(ref p) = s.active {
            return p.scan_id == scan_id && p.status == ScanStatus::Cancelled;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Core scanning logic (runs inside tokio::spawn)
// ---------------------------------------------------------------------------

/// Walk entry collected from the filesystem.
#[derive(Clone, Debug)]
struct WalkEntry {
    file_path: String,
    mtime_ms: i64,
    size_bytes: i64,
}

/// Main scan routine.
async fn run_scan(
    db: &DatabaseConnection,
    scan_state: &Arc<RwLock<ScanState>>,
    scan_id: &str,
    library_ids: &[i32],
) -> Result<(), AppError> {
    tracing::info!("Scan {scan_id}: starting (libraries={library_ids:?})");

    // ---- resolve libraries ----
    let libs = if library_ids.is_empty() {
        libraries::Entity::find().all(db).await?
    } else {
        libraries::Entity::find()
            .filter(libraries::Column::Id.is_in(library_ids.to_vec()))
            .all(db)
            .await?
    };

    if libs.is_empty() {
        return Err(AppError::not_found("No libraries found to scan"));
    }

    // ---- Phase 1: Walk filesystem ----
    let mut all_entries: Vec<(i32, Vec<WalkEntry>)> = Vec::new();
    let mut total_files: i64 = 0;

    for lib in &libs {
        let path = Path::new(&lib.path);
        if !path.exists() {
            tracing::warn!("Scan {scan_id}: library path does not exist: {}", lib.path);
            continue;
        }
        let entries = walk_directory(path, scan_id)?;
        total_files += entries.len() as i64;
        all_entries.push((lib.id, entries));
    }

    // Update total
    {
        let mut s = scan_state.write().await;
        if let Some(ref mut p) = s.active {
            p.files_total = total_files;
        }
    }

    tracing::info!("Scan {scan_id}: walk complete, {total_files} audio files found");

    // ---- Phase 2 & 3: Quick Scan + Full Scan per library ----
    for (lib_id, entries) in &all_entries {
        if is_cancelled(scan_state, scan_id) {
            tracing::info!("Scan {scan_id}: cancelled after walk");
            return Ok(());
        }

        process_library(db, scan_state, scan_id, *lib_id, entries).await?;
    }

    // ---- Phase 4: Cleanup ----
    for (lib_id, entries) in &all_entries {
        if is_cancelled(scan_state, scan_id) {
            tracing::info!("Scan {scan_id}: cancelled before cleanup");
            return Ok(());
        }
        cleanup_orphans(db, *lib_id, entries, scan_id).await?;
    }

    // ---- Done ----
    {
        let mut s = scan_state.write().await;
        if let Some(ref mut p) = s.active {
            if p.scan_id == *scan_id {
                p.status = ScanStatus::Completed;
                p.files_scanned = total_files;
                p.files_total = total_files;
            }
        }
    }

    tracing::info!("Scan {scan_id}: completed successfully");
    Ok(())
}

// ---------------------------------------------------------------------------
// Phase 1: Walk
// ---------------------------------------------------------------------------

/// Recursively walk a directory, collecting audio files.
fn walk_directory(root: &Path, scan_id: &str) -> Result<Vec<WalkEntry>, AppError> {
    let mut entries = Vec::new();
    walk_dir_recursive(root, &mut entries, scan_id)?;
    Ok(entries)
}

fn walk_dir_recursive(dir: &Path, entries: &mut Vec<WalkEntry>, scan_id: &str) -> Result<(), AppError> {
    let read_dir = std::fs::read_dir(dir).map_err(|e| {
        AppError::internal(format!("Failed to read directory {}: {e}", dir.display()))
    })?;

    for entry in read_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };

        if file_type.is_dir() {
            walk_dir_recursive(&path, entries, scan_id)?;
        } else if file_type.is_file() && metadata::is_audio_file(&path) {
            let mtime_ms = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);
            let size_bytes = entry.metadata().ok().map(|m| m.len() as i64).unwrap_or(0);

            entries.push(WalkEntry {
                file_path: path.to_string_lossy().to_string(),
                mtime_ms,
                size_bytes,
            });
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Phase 2 & 3: Quick Scan and Full Scan
// ---------------------------------------------------------------------------

async fn process_library(
    db: &DatabaseConnection,
    scan_state: &Arc<RwLock<ScanState>>,
    scan_id: &str,
    library_id: i32,
    entries: &[WalkEntry],
) -> Result<(), AppError> {
    // Quick Scan: load existing songs for this library
    let existing_songs = songs::Entity::find()
        .filter(songs::Column::LibraryId.eq(library_id))
        .all(db)
        .await?;

    let existing_map: HashMap<String, (i32, i64, i64)> = existing_songs
        .iter()
        .map(|s| {
            // Use created_at as a proxy for last-seen mtime since we don't store mtime separately
            (s.file_path.clone(), (s.id, s.size_bytes.unwrap_or(0), s.created_at))
        })
        .collect();

    let mut fs_paths: HashSet<&str> = HashSet::with_capacity(entries.len());
    let mut to_scan: Vec<&WalkEntry> = Vec::new();

    for entry in entries {
        fs_paths.insert(&entry.file_path);
        match existing_map.get(&entry.file_path) {
            Some((_id, db_size, _db_time)) => {
                // File exists in DB — check if modified (size changed = definitely modified)
                if *db_size != entry.size_bytes {
                    to_scan.push(entry);
                }
                // Note: mtime comparison would be better but SQLite stores created_at
                // not the file system mtime. A future migration could add an `mtime` column.
            }
            None => {
                // New file
                to_scan.push(entry);
            }
        }
    }

    // Update total to reflect actual work
    {
        let mut s = scan_state.write().await;
        if let Some(ref mut p) = s.active {
            p.files_total = to_scan.len() as i64;
        }
    }

    tracing::info!(
        "Scan {scan_id}: library {library_id} — {} total, {} to scan",
        entries.len(),
        to_scan.len()
    );

    // Full Scan: read metadata for each file that needs it
    for (i, entry) in to_scan.iter().enumerate() {
        if is_cancelled(scan_state, scan_id) {
            return Ok(());
        }

        match scan_single_file(db, library_id, entry).await {
            Ok(()) => {}
            Err(e) => {
                tracing::warn!("Scan {scan_id}: failed to scan {}: {e}", entry.file_path);
            }
        }

        // Progress update every 10 files
        if i % 10 == 0 {
            let mut s = scan_state.write().await;
            if let Some(ref mut p) = s.active {
                p.files_scanned = (i + 1) as i64;
            }
        }
    }

    // Final progress update
    {
        let mut s = scan_state.write().await;
        if let Some(ref mut p) = s.active {
            p.files_scanned = to_scan.len() as i64;
        }
    }

    Ok(())
}

/// Process a single audio file: read metadata and upsert into DB.
async fn scan_single_file(
    db: &DatabaseConnection,
    library_id: i32,
    entry: &WalkEntry,
) -> Result<(), AppError> {
    let path = Path::new(&entry.file_path);

    // Read metadata via lofty
    let meta = metadata::read_metadata(path).unwrap_or_else(|| {
        // Bare minimum fallback when tags are unreadable
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown");
        metadata::SongMetadata {
            title: stem.to_string(),
            artist: "Unknown Artist".to_string(),
            album: None,
            album_artist: None,
            track_number: None,
            disc_number: None,
            duration_secs: 0,
            bit_rate: None,
            file_format: path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_string(),
            content_type: metadata::mime_type_for_extension(
                path.extension().and_then(|e| e.to_str()).unwrap_or(""),
            )
            .to_string(),
            year: None,
            has_cover_art: false,
        }
    });

    let now = now_ms();

    // ---- UPSERT artist ----
    let artist_id = upsert_artist(db, &meta.artist, library_id, now).await?;

    // ---- UPSERT album ----
    let album_id = if let Some(ref album_name) = meta.album {
        Some(upsert_album(db, album_name, artist_id, library_id, meta.year, now).await?)
    } else {
        None
    };

    // ---- UPSERT song ----
    let existing = songs::Entity::find()
        .filter(songs::Column::FilePath.eq(&entry.file_path))
        .one(db)
        .await?;

    if let Some(song) = existing {
        // Update existing
        let mut active: songs::ActiveModel = song.into();
        active.title = Set(meta.title);
        active.artist_id = Set(artist_id);
        active.album_id = Set(album_id);
        active.track_number = Set(meta.track_number);
        active.disc_number = Set(meta.disc_number);
        active.duration_secs = Set(meta.duration_secs);
        active.bit_rate = Set(meta.bit_rate);
        active.size_bytes = Set(Some(entry.size_bytes));
        active.file_format = Set(Some(meta.file_format));
        active.content_type = Set(Some(meta.content_type));
        active.year = Set(meta.year);
        active.has_cover_art = Set(if meta.has_cover_art { 1 } else { 0 });
        active.library_id = Set(library_id);
        active.update(db).await?;
    } else {
        // Insert new
        let active = songs::ActiveModel {
            title: Set(meta.title),
            artist_id: Set(artist_id),
            album_id: Set(album_id),
            track_number: Set(meta.track_number),
            disc_number: Set(meta.disc_number),
            duration_secs: Set(meta.duration_secs),
            bit_rate: Set(meta.bit_rate),
            size_bytes: Set(Some(entry.size_bytes)),
            file_format: Set(Some(meta.file_format)),
            content_type: Set(Some(meta.content_type)),
            year: Set(meta.year),
            file_path: Set(entry.file_path.clone()),
            has_cover_art: Set(if meta.has_cover_art { 1 } else { 0 }),
            library_id: Set(library_id),
            created_at: Set(now),
            ..Default::default()
        };
        active.insert(db).await?;
    }

    // ---- Extract cover art if present ----
    if meta.has_cover_art {
        if let Some(cover) = metadata::extract_cover(path) {
            // Upsert cover_art for the song
            let existing_cover = cover_art::Entity::find()
                .filter(cover_art::Column::ItemType.eq("song"))
                .filter(cover_art::Column::ItemId.eq(
                    // We need the song id — let's get it
                    songs::Entity::find()
                        .filter(songs::Column::FilePath.eq(&entry.file_path))
                        .one(db)
                        .await?
                        .map(|s| s.id)
                        .unwrap_or(0)
                ))
                .one(db)
                .await?;

            if let Some(c) = existing_cover {
                let mut active: cover_art::ActiveModel = c.into();
                active.mime_type = Set(cover.mime_type.clone());
                active.width = Set(cover.width);
                active.height = Set(cover.height);
                active.file_path = Set(None); // embedded, no external file
                active.update(db).await?;
            } else {
                // Insert — but we need the song id first
                if let Some(song) = songs::Entity::find()
                    .filter(songs::Column::FilePath.eq(&entry.file_path))
                    .one(db)
                    .await?
                {
                    let active = cover_art::ActiveModel {
                        item_type: Set("song".to_string()),
                        item_id: Set(song.id),
                        mime_type: Set(cover.mime_type.clone()),
                        width: Set(cover.width),
                        height: Set(cover.height),
                        file_path: Set(None),
                        ..Default::default()
                    };
                    active.insert(db).await?;
                }
            }
        }
    }

    Ok(())
}

/// Upsert an artist by (name, library_id). Returns the artist id.
async fn upsert_artist(
    db: &DatabaseConnection,
    name: &str,
    library_id: i32,
    now: i64,
) -> Result<i32, AppError> {
    let existing = artists::Entity::find()
        .filter(artists::Column::Name.eq(name))
        .filter(artists::Column::LibraryId.eq(library_id))
        .one(db)
        .await?;

    if let Some(a) = existing {
        Ok(a.id)
    } else {
        let active = artists::ActiveModel {
            name: Set(name.to_string()),
            sort_name: Set(Some(sort_name(name))),
            library_id: Set(library_id),
            created_at: Set(now),
            ..Default::default()
        };
        let res = active.insert(db).await?;
        Ok(res.id)
    }
}

/// Upsert an album by (name, artist_id, library_id). Returns the album id.
async fn upsert_album(
    db: &DatabaseConnection,
    name: &str,
    artist_id: i32,
    library_id: i32,
    year: Option<i32>,
    now: i64,
) -> Result<i32, AppError> {
    let existing = albums::Entity::find()
        .filter(albums::Column::Name.eq(name))
        .filter(albums::Column::ArtistId.eq(artist_id))
        .filter(albums::Column::LibraryId.eq(library_id))
        .one(db)
        .await?;

    if let Some(a) = existing {
        Ok(a.id)
    } else {
        let active = albums::ActiveModel {
            name: Set(name.to_string()),
            artist_id: Set(artist_id),
            library_id: Set(library_id),
            year: Set(year),
            created_at: Set(now),
            ..Default::default()
        };
        let res = active.insert(db).await?;
        Ok(res.id)
    }
}

/// Normalise a name for sorting: strip leading "The ", "A ", "An ".
fn sort_name(name: &str) -> String {
    let lower = name.to_lowercase();
    for prefix in &["the ", "a ", "an "] {
        if lower.starts_with(prefix) {
            return name[prefix.len()..].trim().to_string();
        }
    }
    name.to_string()
}

// ---------------------------------------------------------------------------
// Phase 4: Cleanup orphans
// ---------------------------------------------------------------------------

async fn cleanup_orphans(
    db: &DatabaseConnection,
    library_id: i32,
    entries: &[WalkEntry],
    scan_id: &str,
) -> Result<(), AppError> {
    let fs_paths: HashSet<&str> = entries.iter().map(|e| e.file_path.as_str()).collect();

    // Find songs in DB that no longer exist on disk
    let db_songs = songs::Entity::find()
        .filter(songs::Column::LibraryId.eq(library_id))
        .all(db)
        .await?;

    let mut orphan_ids = Vec::new();
    for song in &db_songs {
        if !fs_paths.contains(song.file_path.as_str()) {
            orphan_ids.push(song.id);
        }
    }

    if orphan_ids.is_empty() {
        tracing::info!("Scan {scan_id}: library {library_id} — no orphans");
        return Ok(());
    }

    tracing::info!(
        "Scan {scan_id}: library {library_id} — removing {} orphan songs",
        orphan_ids.len()
    );

    // Delete orphan songs
    for chunk in orphan_ids.chunks(100) {
        songs::Entity::delete_many()
            .filter(songs::Column::Id.is_in(chunk.to_vec()))
            .exec(db)
            .await?;
    }

    // Clean up artists with no remaining songs in this library
    let remaining_artist_ids: HashSet<i32> = songs::Entity::find()
        .filter(songs::Column::LibraryId.eq(library_id))
        .all(db)
        .await?
        .iter()
        .map(|s| s.artist_id)
        .collect();

    let all_artists = artists::Entity::find()
        .filter(artists::Column::LibraryId.eq(library_id))
        .all(db)
        .await?;

    for artist in &all_artists {
        if !remaining_artist_ids.contains(&artist.id) {
            artists::Entity::delete_by_id(artist.id).exec(db).await?;
        }
    }

    // Clean up albums with no remaining songs in this library
    let remaining_album_ids: HashSet<i32> = songs::Entity::find()
        .filter(songs::Column::LibraryId.eq(library_id))
        .filter(songs::Column::AlbumId.is_not_null())
        .all(db)
        .await?
        .iter()
        .filter_map(|s| s.album_id)
        .collect();

    let all_albums = albums::Entity::find()
        .filter(albums::Column::LibraryId.eq(library_id))
        .all(db)
        .await?;

    for album in &all_albums {
        if !remaining_album_ids.contains(&album.id) {
            albums::Entity::delete_by_id(album.id).exec(db).await?;
        }
    }

    tracing::info!("Scan {scan_id}: library {library_id} — cleanup complete");
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
