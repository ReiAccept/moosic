use rand::RngExt as _;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::Serialize;

use crate::utils::now_ms;
use crate::entities::prelude::*;
use crate::entities::shares;
use crate::error::AppError;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Details returned when resolving a share token — includes the share metadata
/// and the serialised underlying item (song / album / playlist).
#[derive(Serialize)]
pub struct ShareDetail {
    pub share: ShareItem,
    pub item: serde_json::Value,
}

/// Metadata for a single share.
#[derive(Serialize, Clone)]
pub struct ShareItem {
    pub id: i32,
    pub r#type: String,
    pub item_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub token: String,
    pub url: String,
    pub visit_count: i32,
    pub last_visited_at: Option<i64>,
    pub expires_at: Option<i64>,
    pub created_at: i64,
}

/// Generate a random 32-character alphanumeric token.
fn generate_share_token() -> String {
    let mut rng = rand::rng();
    (&mut rng)
        .sample_iter(&rand::distr::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

/// Look up the human-readable title for the given item type and id.
async fn resolve_title(
    db: &DatabaseConnection,
    item_type: &str,
    item_id: i32,
) -> Result<String, AppError> {
    match item_type {
        "song" => {
            let song = SongEntity::find_by_id(item_id)
                .one(db)
                .await?
                .ok_or_else(|| AppError::not_found("Song not found"))?;
            Ok(song.title)
        }
        "album" => {
            let album = AlbumEntity::find_by_id(item_id)
                .one(db)
                .await?
                .ok_or_else(|| AppError::not_found("Album not found"))?;
            Ok(album.name)
        }
        "playlist" => {
            let playlist = PlaylistEntity::find_by_id(item_id)
                .one(db)
                .await?
                .ok_or_else(|| AppError::not_found("Playlist not found"))?;
            Ok(playlist.name)
        }
        _ => Err(AppError::validation_error(format!(
            "Unknown item type: {item_type}"
        ))),
    }
}

/// Build a `ShareItem` from a share model, resolving the item's title.
async fn share_to_item(
    db: &DatabaseConnection,
    share: &shares::Model,
) -> Result<ShareItem, AppError> {
    let title = resolve_title(db, &share.item_type, share.item_id).await?;
    Ok(ShareItem {
        id: share.id,
        r#type: share.item_type.clone(),
        item_id: share.item_id,
        title,
        description: share.description.clone(),
        token: share.token.clone(),
        url: format!("/api/share/{}", share.token),
        visit_count: share.visit_count,
        last_visited_at: share.last_visited_at,
        expires_at: share.expires_at,
        created_at: share.created_at,
    })
}

/// Validate that `item_type` is one of the known types.
fn validate_item_type(item_type: &str) -> Result<(), AppError> {
    match item_type {
        "song" | "album" | "playlist" => Ok(()),
        _ => Err(AppError::validation_error(format!(
            "Invalid item type '{item_type}'. Must be one of: song, album, playlist"
        ))),
    }
}

// ---------------------------------------------------------------------------
// Operations
// ---------------------------------------------------------------------------

/// Resolve a share by its token, increment the visit counter, and return the
/// share detail together with the serialised item.
pub async fn get_by_token(
    db: &DatabaseConnection,
    token: &str,
) -> Result<ShareDetail, AppError> {
    let share = ShareEntity::find()
        .filter(shares::Column::Token.eq(token))
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Share not found"))?;

    let now = now_ms();

    // Check expiry.
    if let Some(expires) = share.expires_at {
        if expires < now {
            return Err(AppError::gone("Share has expired"));
        }
    }

    // Increment visit count and update last_visited_at.
    let visit_count = share.visit_count;
    let mut active: shares::ActiveModel = share.into();
    active.visit_count = ActiveValue::Set(visit_count + 1);
    active.last_visited_at = ActiveValue::Set(Some(now));
    let updated = active.update(db).await?;

    // Fetch the referenced item and serialise it.
    let item = match updated.item_type.as_str() {
        "song" => {
            let song = SongEntity::find_by_id(updated.item_id)
                .one(db)
                .await?
                .ok_or_else(|| AppError::not_found("Shared song not found"))?;
            serde_json::to_value(&song).unwrap_or(serde_json::Value::Null)
        }
        "album" => {
            let album = AlbumEntity::find_by_id(updated.item_id)
                .one(db)
                .await?
                .ok_or_else(|| AppError::not_found("Shared album not found"))?;
            serde_json::to_value(&album).unwrap_or(serde_json::Value::Null)
        }
        "playlist" => {
            let playlist = PlaylistEntity::find_by_id(updated.item_id)
                .one(db)
                .await?
                .ok_or_else(|| AppError::not_found("Shared playlist not found"))?;
            serde_json::to_value(&playlist).unwrap_or(serde_json::Value::Null)
        }
        _ => serde_json::Value::Null,
    };

    // Build the share metadata for the response.
    let title = resolve_title(db, &updated.item_type, updated.item_id).await?;
    let token = updated.token.clone();
    let share_item = ShareItem {
        id: updated.id,
        r#type: updated.item_type,
        item_id: updated.item_id,
        title,
        description: updated.description,
        token,
        url: format!("/api/share/{}", updated.token),
        visit_count: updated.visit_count,
        last_visited_at: updated.last_visited_at,
        expires_at: updated.expires_at,
        created_at: updated.created_at,
    };

    Ok(ShareDetail {
        share: share_item,
        item,
    })
}

/// List all shares belonging to a user.
pub async fn list_for_user(
    db: &DatabaseConnection,
    user_id: i32,
) -> Result<Vec<ShareItem>, AppError> {
    let share_models = ShareEntity::find()
        .filter(shares::Column::OwnerId.eq(user_id))
        .order_by_desc(shares::Column::CreatedAt)
        .all(db)
        .await?;

    let mut items = Vec::with_capacity(share_models.len());
    for share in &share_models {
        items.push(share_to_item(db, share).await?);
    }

    Ok(items)
}

/// Create a new share for a song, album, or playlist.
///
/// Generates a unique token and optionally sets an expiration date.
/// The `expires_in_days` parameter, when provided, sets the share to expire
/// that many days from now.
pub async fn create(
    db: &DatabaseConnection,
    owner_id: i32,
    item_type: &str,
    item_id: i32,
    description: Option<&str>,
    expires_in_days: Option<i32>,
) -> Result<ShareItem, AppError> {
    validate_item_type(item_type)?;

    // Verify the referenced item exists.
    match item_type {
        "song" => {
            SongEntity::find_by_id(item_id)
                .one(db)
                .await?
                .ok_or_else(|| AppError::not_found("Song not found"))?;
        }
        "album" => {
            AlbumEntity::find_by_id(item_id)
                .one(db)
                .await?
                .ok_or_else(|| AppError::not_found("Album not found"))?;
        }
        "playlist" => {
            PlaylistEntity::find_by_id(item_id)
                .one(db)
                .await?
                .ok_or_else(|| AppError::not_found("Playlist not found"))?;
        }
        _ => unreachable!(),
    }

    let now = now_ms();
    let token = generate_share_token();

    let expires_at = expires_in_days.map(|days| now + days as i64 * 86400 * 1000);

    let active = shares::ActiveModel {
        id: ActiveValue::NotSet,
        owner_id: ActiveValue::Set(owner_id),
        item_type: ActiveValue::Set(item_type.to_string()),
        item_id: ActiveValue::Set(item_id),
        description: ActiveValue::Set(description.map(|s| s.to_string())),
        token: ActiveValue::Set(token),
        visit_count: ActiveValue::Set(0),
        last_visited_at: ActiveValue::Set(None),
        expires_at: ActiveValue::Set(expires_at),
        created_at: ActiveValue::Set(now),
    };

    let inserted = active.insert(db).await?;
    share_to_item(db, &inserted).await
}

/// Update a share's description and/or expiration.
///
/// - `description`:
///   - `None` — leave the description unchanged.
///   - `Some(None)` — clear the description (set to `None`).
///   - `Some(Some(s))` — set description to `s`.
/// - `expires_in_days`:
///   - `None` — leave expiration unchanged.
///   - `Some(d)` — recompute `expires_at` from now.
pub async fn update(
    db: &DatabaseConnection,
    id: i32,
    owner_id: i32,
    description: Option<Option<String>>,
    expires_in_days: Option<i32>,
) -> Result<ShareItem, AppError> {
    let share = ShareEntity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Share not found"))?;

    if share.owner_id != owner_id {
        return Err(AppError::forbidden("Cannot update another user's share"));
    }

    let mut active: shares::ActiveModel = share.into();

    if let Some(desc) = description {
        active.description = ActiveValue::Set(desc);
    }

    if let Some(days) = expires_in_days {
        active.expires_at = ActiveValue::Set(Some(now_ms() + days as i64 * 86400 * 1000));
    }

    let updated = active.update(db).await?;
    share_to_item(db, &updated).await
}

/// Delete a single share, verifying ownership (admin bypass).
pub async fn delete(
    db: &DatabaseConnection,
    id: i32,
    owner_id: i32,
    is_admin: bool,
) -> Result<(), AppError> {
    let share = ShareEntity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Share not found"))?;

    if share.owner_id != owner_id && !is_admin {
        return Err(AppError::forbidden("Cannot delete another user's share"));
    }

    ShareEntity::delete_by_id(share.id).exec(db).await?;
    Ok(())
}

/// Delete multiple shares, verifying ownership for each (admin bypass).
/// Returns the number of shares actually deleted.
pub async fn delete_batch(
    db: &DatabaseConnection,
    ids: &[i32],
    owner_id: i32,
    is_admin: bool,
) -> Result<u64, AppError> {
    let shares = ShareEntity::find()
        .filter(shares::Column::Id.is_in(ids.to_vec()))
        .all(db)
        .await?;

    let valid_ids: Vec<i32> = shares
        .into_iter()
        .filter(|s| s.owner_id == owner_id || is_admin)
        .map(|s| s.id)
        .collect();

    if valid_ids.is_empty() {
        return Ok(0);
    }

    let result = ShareEntity::delete_many()
        .filter(shares::Column::Id.is_in(valid_ids))
        .exec(db)
        .await?;

    Ok(result.rows_affected)
}
