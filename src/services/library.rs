use std::time::{SystemTime, UNIX_EPOCH};

use sea_orm::{
    ActiveModelTrait, ActiveValue::*, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, PaginatorTrait, QueryFilter,
};
use serde::Serialize;

use crate::entities::{albums, artists, libraries, songs, user_libraries};
use crate::error::AppError;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// 13-digit Unix millisecond timestamp.
fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A library with per-user enabled status and song count.
#[derive(Serialize)]
pub struct LibraryWithStatus {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub is_enabled: bool,
    pub song_count: i64,
    pub created_at: i64,
}

/// Aggregate counts returned when a library is deleted.
pub struct DeleteLibraryCounts {
    pub songs: u64,
    pub albums: u64,
    pub artists: u64,
}

// ---------------------------------------------------------------------------
// User-facing library queries
// ---------------------------------------------------------------------------

/// List all libraries with per-user enabled status and song counts.
pub async fn list_for_user(
    db: &DatabaseConnection,
    user_id: i32,
) -> Result<Vec<LibraryWithStatus>, AppError> {
    let all_libraries = libraries::Entity::find().all(db).await?;

    // Batch-fetch per-user library enabled status.
    let user_libs = user_libraries::Entity::find()
        .filter(user_libraries::Column::UserId.eq(user_id))
        .all(db)
        .await?;

    let mut result = Vec::with_capacity(all_libraries.len());

    for lib in all_libraries {
        let is_enabled = user_libs
            .iter()
            .find(|ul| ul.library_id == lib.id)
            .map(|ul| ul.is_enabled != 0)
            .unwrap_or(true); // default to enabled if no row

        let song_count = songs::Entity::find()
            .filter(songs::Column::LibraryId.eq(lib.id))
            .count(db)
            .await? as i64;

        result.push(LibraryWithStatus {
            id: lib.id,
            name: lib.name,
            path: lib.path,
            is_enabled,
            song_count,
            created_at: lib.created_at,
        });
    }

    Ok(result)
}

/// Enable a library for a specific user (upsert).
pub async fn enable_for_user(
    db: &DatabaseConnection,
    user_id: i32,
    library_id: i32,
) -> Result<(), AppError> {
    set_user_library_enabled(db, user_id, library_id, 1).await
}

/// Disable a library for a specific user (upsert).
pub async fn disable_for_user(
    db: &DatabaseConnection,
    user_id: i32,
    library_id: i32,
) -> Result<(), AppError> {
    set_user_library_enabled(db, user_id, library_id, 0).await
}

/// Internal helper to upsert a user_library row.
async fn set_user_library_enabled(
    db: &DatabaseConnection,
    user_id: i32,
    library_id: i32,
    enabled: i32,
) -> Result<(), AppError> {
    let existing = user_libraries::Entity::find()
        .filter(user_libraries::Column::UserId.eq(user_id))
        .filter(user_libraries::Column::LibraryId.eq(library_id))
        .one(db)
        .await?;

    if let Some(ul) = existing {
        let mut ac = ul.into_active_model();
        ac.is_enabled = Set(enabled);
        ac.update(db).await?;
    } else {
        user_libraries::ActiveModel {
            user_id: Set(user_id),
            library_id: Set(library_id),
            is_enabled: Set(enabled),
        }
        .insert(db)
        .await?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Library CRUD (admin)
// ---------------------------------------------------------------------------

/// Create a new library. Returns a conflict error if the path already exists.
pub async fn add_library(
    db: &DatabaseConnection,
    name: &str,
    path: &str,
) -> Result<libraries::Model, AppError> {
    // Check path uniqueness.
    let existing = libraries::Entity::find()
        .filter(libraries::Column::Path.eq(path))
        .one(db)
        .await?;
    if existing.is_some() {
        return Err(AppError::conflict("Library path already exists"));
    }

    let now = now_ms();

    let new_library = libraries::ActiveModel {
        id: NotSet,
        name: Set(name.to_string()),
        path: Set(path.to_string()),
        is_enabled: Set(1),
        watch_enabled: Set(0),
        created_at: Set(now),
    };

    let result = libraries::Entity::insert(new_library).exec(db).await?;
    let inserted = libraries::Entity::find_by_id(result.last_insert_id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::internal("Failed to retrieve created library"))?;

    Ok(inserted)
}

/// Delete a library and return counts of associated songs, albums, and artists.
pub async fn delete_library(
    db: &DatabaseConnection,
    library_id: i32,
) -> Result<DeleteLibraryCounts, AppError> {
    let song_count = songs::Entity::find()
        .filter(songs::Column::LibraryId.eq(library_id))
        .count(db)
        .await?;

    let album_count = albums::Entity::find()
        .filter(albums::Column::LibraryId.eq(library_id))
        .count(db)
        .await?;

    let artist_count = artists::Entity::find()
        .filter(artists::Column::LibraryId.eq(library_id))
        .count(db)
        .await?;

    libraries::Entity::delete_by_id(library_id).exec(db).await?;

    Ok(DeleteLibraryCounts {
        songs: song_count,
        albums: album_count,
        artists: artist_count,
    })
}

/// Update a library's name and/or path. Returns a conflict error if the new
/// path is already taken by a different library.
pub async fn update_library(
    db: &DatabaseConnection,
    id: i32,
    name: Option<String>,
    path: Option<String>,
) -> Result<libraries::Model, AppError> {
    let target = libraries::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Library not found"))?;

    // If the path is being changed, check uniqueness.
    if let Some(ref new_path) = path {
        if *new_path != target.path {
            let existing = libraries::Entity::find()
                .filter(libraries::Column::Path.eq(new_path))
                .one(db)
                .await?;
            if existing.is_some() {
                return Err(AppError::conflict("Library path already exists"));
            }
        }
    }

    let mut active_model = target.into_active_model();

    if let Some(name) = name {
        active_model.name = Set(name);
    }
    if let Some(path) = path {
        active_model.path = Set(path);
    }

    let updated = active_model.update(db).await?;
    Ok(updated)
}

/// Enable filesystem watching for a library.
pub async fn enable_notify(
    db: &DatabaseConnection,
    library_id: i32,
) -> Result<(), AppError> {
    let target = libraries::Entity::find_by_id(library_id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Library not found"))?;

    let mut active_model = target.into_active_model();
    active_model.watch_enabled = Set(1);
    active_model.update(db).await?;

    Ok(())
}

/// Disable filesystem watching for a library.
pub async fn disable_notify(
    db: &DatabaseConnection,
    library_id: i32,
) -> Result<(), AppError> {
    let target = libraries::Entity::find_by_id(library_id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Library not found"))?;

    let mut active_model = target.into_active_model();
    active_model.watch_enabled = Set(0);
    active_model.update(db).await?;

    Ok(())
}

/// Get a single library by ID, or return not_found.
pub async fn get_by_id(
    db: &DatabaseConnection,
    id: i32,
) -> Result<libraries::Model, AppError> {
    libraries::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Library not found"))
}
