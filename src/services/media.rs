use crate::entities::prelude::*;
use crate::entities::user_libraries;
use crate::error::AppError;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tokio::fs;

/// Information needed to stream a song to a client.
pub struct StreamInfo {
    pub data: Vec<u8>,
    pub content_type: String,
    pub content_length: u64,
    pub file_name: Option<String>,
}

/// Get audio file data for streaming, verifying that the user has access to
/// the library that owns the song.
pub async fn get_stream_data(
    db: &DatabaseConnection,
    user_id: i32,
    song_id: i32,
) -> Result<StreamInfo, AppError> {
    // Look up the song.
    let song = SongEntity::find_by_id(song_id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Song not found"))?;

    // Verify library access.
    let has_access = check_library_access(db, user_id, song.library_id).await?;
    if !has_access {
        return Err(AppError::forbidden("Access denied to this song's library"));
    }

    // Read the file from disk.
    let data = read_audio_file(&song.file_path).await?;

    // Build a human-friendly download filename "Artist - Title.ext".
    let artist = ArtistEntity::find_by_id(song.artist_id)
        .one(db)
        .await?
        .map(|a| a.name)
        .unwrap_or_default();

    let ext = song.file_format.as_deref().unwrap_or("mp3");
    let file_name = Some(format!("{} - {}.{}", artist, song.title, ext));

    let content_type = song.content_type.unwrap_or_else(|| "audio/mpeg".to_string());
    let content_length = data.len() as u64;

    Ok(StreamInfo {
        data,
        content_type,
        content_length,
        file_name,
    })
}

/// Verify that a user has been granted access to a specific library.
///
/// Returns `true` when a `user_libraries` row exists with `is_enabled ≠ 0`.
/// A missing or disabled row returns `false`.
pub async fn check_library_access(
    db: &DatabaseConnection,
    user_id: i32,
    library_id: i32,
) -> Result<bool, AppError> {
    let access = UserLibraryEntity::find()
        .filter(user_libraries::Column::UserId.eq(user_id))
        .filter(user_libraries::Column::LibraryId.eq(library_id))
        .one(db)
        .await?;

    match access {
        Some(ul) if ul.is_enabled != 0 => Ok(true),
        _ => Ok(false),
    }
}

/// Read a file from disk, returning a `not_found` error when the file is
/// missing or inaccessible.
pub async fn read_audio_file(path: &str) -> Result<Vec<u8>, AppError> {
    fs::read(path)
        .await
        .map_err(|_| AppError::not_found("File not found on disk"))
}
