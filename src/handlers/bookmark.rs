use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::middleware::auth::AuthenticatedUser;
use crate::services;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct BookmarkGetRequest {
    pub song_id: i32,
    pub device_id: Option<String>,
}

#[derive(Deserialize)]
pub struct BookmarkCreateRequest {
    pub song_id: i32,
    pub position_ms: i32,
    pub device_id: Option<String>,
}

// Re-using IdRequest from playlist module or defining locally
#[derive(Deserialize)]
pub struct IdRequest {
    pub id: i32,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct BookmarkResponse {
    pub id: i32,
    pub user_id: i32,
    pub song_id: i32,
    pub position_ms: i32,
    pub device_id: Option<String>,
    pub updated_at: i64,
    pub song_title: Option<String>,
    pub artist_name: Option<String>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/bookmark/list
pub async fn bookmark_list(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let items = services::bookmark::list(&state.db, user.id).await?;
    Ok(Json(serde_json::json!({"bookmarks": items})))
}

/// POST /api/bookmark/get
pub async fn bookmark_get(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<BookmarkGetRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let bookmark = services::bookmark::get(
        &state.db,
        user.id,
        payload.song_id,
        payload.device_id.as_deref(),
    )
    .await?
    .ok_or_else(|| AppError::not_found("Bookmark not found"))?;

    Ok(Json(serde_json::json!({
        "bookmark": {
            "id": bookmark.id,
            "user_id": bookmark.user_id,
            "song_id": bookmark.song_id,
            "position_ms": bookmark.position_ms,
            "device_id": bookmark.device_id,
            "updated_at": bookmark.updated_at,
        }
    })))
}

/// POST /api/bookmark/create
pub async fn bookmark_create(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<BookmarkCreateRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    services::bookmark::upsert(
        &state.db,
        user.id,
        payload.song_id,
        payload.position_ms,
        payload.device_id.as_deref(),
    )
    .await?;

    Ok(Json(serde_json::json!({"status": "ok"})))
}

/// POST /api/bookmark/delete
pub async fn bookmark_delete(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<IdRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    services::bookmark::delete(&state.db, user.id, payload.id).await?;

    Ok(Json(serde_json::json!({"message": "Bookmark deleted"})))
}
