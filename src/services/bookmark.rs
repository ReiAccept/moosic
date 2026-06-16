use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::Serialize;

use crate::entities::prelude::*;
use crate::entities::bookmarks;
use crate::error::AppError;
use crate::utils::now_ms;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct BookmarkItem {
    pub id: i32,
    pub song_id: i32,
    pub title: String,
    pub artist_name: String,
    pub position_ms: i32,
    pub device_id: Option<String>,
    pub updated_at: i64,
}


// ---------------------------------------------------------------------------
// CRUD operations
// ---------------------------------------------------------------------------

/// List all bookmarks for a user, chronologically by updated_at DESC.
/// Each item resolves the song title and artist name.
pub async fn list(
    db: &DatabaseConnection,
    user_id: i32,
) -> Result<Vec<BookmarkItem>, AppError> {
    let bookmarks = BookmarkEntity::find()
        .filter(bookmarks::Column::UserId.eq(user_id))
        .order_by_desc(bookmarks::Column::UpdatedAt)
        .all(db)
        .await?;

    let mut items = Vec::with_capacity(bookmarks.len());
    for bm in bookmarks {
        let song = SongEntity::find_by_id(bm.song_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::not_found(format!("Song {} not found", bm.song_id)))?;

        let artist = ArtistEntity::find_by_id(song.artist_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::not_found(format!("Artist {} not found", song.artist_id)))?;

        items.push(BookmarkItem {
            id: bm.id,
            song_id: bm.song_id,
            title: song.title,
            artist_name: artist.name,
            position_ms: bm.position_ms,
            device_id: bm.device_id,
            updated_at: bm.updated_at,
        });
    }

    Ok(items)
}

/// Get a single bookmark by user + song + optional device.
/// Returns `None` when no matching bookmark exists.
pub async fn get(
    db: &DatabaseConnection,
    user_id: i32,
    song_id: i32,
    device_id: Option<&str>,
) -> Result<Option<bookmarks::Model>, AppError> {
    let mut query = BookmarkEntity::find()
        .filter(bookmarks::Column::UserId.eq(user_id))
        .filter(bookmarks::Column::SongId.eq(song_id));

    match device_id {
        Some(did) => {
            query = query.filter(bookmarks::Column::DeviceId.eq(did));
        }
        None => {
            query = query.filter(bookmarks::Column::DeviceId.is_null());
        }
    }

    Ok(query.one(db).await?)
}

/// Create or update a bookmark for a specific song+device.
///
/// Validates that the song exists and that `position_ms` does not exceed
/// the song's duration. Returns the (updated or newly inserted) model.
pub async fn upsert(
    db: &DatabaseConnection,
    user_id: i32,
    song_id: i32,
    position_ms: i32,
    device_id: Option<&str>,
) -> Result<bookmarks::Model, AppError> {
    // Validate song exists.
    let song = SongEntity::find_by_id(song_id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Song not found"))?;

    // Validate position does not exceed song duration.
    if position_ms >= song.duration_secs * 1000 {
        return Err(AppError::validation_error(format!(
            "Position {}ms exceeds song duration {}s",
            position_ms, song.duration_secs
        )));
    }

    // Locate an existing bookmark for this user+song+device combination.
    let existing = {
        let mut q = BookmarkEntity::find()
            .filter(bookmarks::Column::UserId.eq(user_id))
            .filter(bookmarks::Column::SongId.eq(song_id));

        match device_id {
            Some(did) => {
                q = q.filter(bookmarks::Column::DeviceId.eq(did));
            }
            None => {
                q = q.filter(bookmarks::Column::DeviceId.is_null());
            }
        }

        q.one(db).await?
    };

    let now = now_ms();

    if let Some(existing_model) = existing {
        let mut active: bookmarks::ActiveModel = existing_model.into();
        active.position_ms = ActiveValue::Set(position_ms);
        active.updated_at = ActiveValue::Set(now);
        Ok(active.update(db).await?)
    } else {
        let active = bookmarks::ActiveModel {
            id: ActiveValue::NotSet,
            user_id: ActiveValue::Set(user_id),
            song_id: ActiveValue::Set(song_id),
            position_ms: ActiveValue::Set(position_ms),
            device_id: ActiveValue::Set(device_id.map(|s| s.to_string())),
            updated_at: ActiveValue::Set(now),
        };
        Ok(active.insert(db).await?)
    }
}

/// Delete a bookmark by its primary key, verifying ownership.
pub async fn delete(
    db: &DatabaseConnection,
    user_id: i32,
    bookmark_id: i32,
) -> Result<(), AppError> {
    let bookmark = BookmarkEntity::find_by_id(bookmark_id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Bookmark not found"))?;

    if bookmark.user_id != user_id {
        return Err(AppError::forbidden("Cannot delete another user's bookmark"));
    }

    BookmarkEntity::delete_by_id(bookmark.id).exec(db).await?;
    Ok(())
}
