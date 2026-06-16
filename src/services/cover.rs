use crate::entities::prelude::*;
use crate::entities::cover_art;
use crate::error::AppError;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::path::Path;
use tokio::fs;

/// Default placeholder SVG cover art displayed when no image is found.
pub const DEFAULT_COVER_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 300 300"><rect fill="#1a1a2e" width="300" height="300"/><text fill="#e0e0e0" font-size="48" x="150" y="160" text-anchor="middle">♪</text></svg>"##;

/// Get cover art for an item (song / album / artist).
///
/// The lookup follows a fallback chain:
/// 1. The item's own `cover_art` record (file on disk).
/// 2. For songs: the album cover, then the artist cover.
/// 3. For albums: the artist cover.
/// 4. The default SVG placeholder.
///
/// Returns `(mime_type, data)`.
pub async fn get_cover(
    db: &DatabaseConnection,
    item_type: &str,
    item_id: i32,
) -> Result<(String, Vec<u8>), AppError> {
    // ---- 1. Check for a dedicated cover_art record ----
    if let Some(result) = try_cover_from_db(db, item_type, item_id).await? {
        return Ok(result);
    }

    // ---- 2. Fallback chain ----
    match item_type {
        "song" => {
            let song = SongEntity::find_by_id(item_id).one(db).await?;
            if let Some(song) = song {
                // Try album cover first
                if let Some(album_id) = song.album_id {
                    if let Some(result) = try_cover_from_db(db, "album", album_id).await? {
                        return Ok(result);
                    }
                }
                // Then try artist cover
                if let Some(result) = try_cover_from_db(db, "artist", song.artist_id).await? {
                    return Ok(result);
                }
            }
        }
        "album" => {
            let album = AlbumEntity::find_by_id(item_id).one(db).await?;
            if let Some(album) = album {
                if let Some(result) = try_cover_from_db(db, "artist", album.artist_id).await? {
                    return Ok(result);
                }
            }
        }
        // "artist" and unknown types have no further fallback.
        _ => {}
    }

    // ---- 3. Default placeholder ----
    Ok((
        "image/svg+xml".to_string(),
        DEFAULT_COVER_SVG.as_bytes().to_vec(),
    ))
}

/// Query the `cover_art` table and read the file from disk if a file path
/// is stored. Returns `None` when no matching record exists or the file
/// is not available on disk.
async fn try_cover_from_db(
    db: &DatabaseConnection,
    item_type: &str,
    item_id: i32,
) -> Result<Option<(String, Vec<u8>)>, AppError> {
    let cover = CoverArtEntity::find()
        .filter(cover_art::Column::ItemType.eq(item_type))
        .filter(cover_art::Column::ItemId.eq(item_id))
        .one(db)
        .await?;

    match cover {
        Some(c) if c.file_path.as_ref().map_or(false, |p| Path::new(p).exists()) => {
            // SAFETY: we just verified file_path is Some and the file exists
            let data = fs::read(c.file_path.as_ref().unwrap())
                .await
                .map_err(|_| AppError::internal("Failed to read cover art file"))?;
            Ok(Some((c.mime_type, data)))
        }
        _ => Ok(None),
    }
}

/// Convenience wrapper around [`get_cover`] that returns the MIME type and
/// raw bytes directly -- useful when building an HTTP response.
pub async fn get_cover_response(
    db: &DatabaseConnection,
    item_type: &str,
    item_id: i32,
) -> Result<(String, Vec<u8>), AppError> {
    get_cover(db, item_type, item_id).await
}
