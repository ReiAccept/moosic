use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::middleware::auth::AuthenticatedUser;
use crate::services;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateShareRequest {
    #[serde(rename = "type")]
    pub r#type: String,
    pub item_id: i32,
    pub description: Option<String>,
    pub expires_in_days: Option<i32>,
}

#[derive(Deserialize)]
pub struct UpdateShareRequest {
    pub id: i32,
    pub description: Option<Option<String>>,
    pub expires_in_days: Option<i32>,
}

#[derive(Deserialize)]
pub struct IdRequest {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct BatchDeleteRequest {
    pub ids: Vec<i32>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct ShareEntry {
    pub id: i32,
    pub owner_id: i32,
    pub item_type: String,
    pub item_id: i32,
    pub description: Option<String>,
    pub token: String,
    pub url: String,
    pub visit_count: i32,
    pub last_visited_at: Option<i64>,
    pub expires_at: Option<i64>,
    pub created_at: i64,
    pub title: Option<String>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/share/{token} — PUBLIC endpoint
pub async fn get_share(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let detail = services::share::get_by_token(&state.db, &token).await?;

    Ok(Json(serde_json::json!({
        "share": detail.share,
        "item": detail.item,
    })))
}

/// POST /api/share/list
pub async fn share_list(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let shares = services::share::list_for_user(&state.db, user.id).await?;

    Ok(Json(serde_json::json!({"shares": shares})))
}

/// POST /api/share/create
pub async fn share_create(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateShareRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let share = services::share::create(
        &state.db,
        user.id,
        &payload.r#type,
        payload.item_id,
        payload.description.as_deref(),
        payload.expires_in_days,
    )
    .await?;

    Ok(Json(serde_json::json!({"share": share})))
}

/// POST /api/share/update
pub async fn share_update(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<UpdateShareRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    services::share::update(
        &state.db,
        payload.id,
        user.id,
        payload.description,
        payload.expires_in_days,
    )
    .await?;

    Ok(Json(serde_json::json!({"message": "Share updated"})))
}

/// POST /api/share/delete
pub async fn share_delete(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<IdRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let is_admin = user
        .privs
        .get("edit_library")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    services::share::delete(&state.db, payload.id, user.id, is_admin).await?;

    Ok(Json(serde_json::json!({"message": "Share deleted"})))
}

/// POST /api/share/delete-batch
pub async fn share_delete_batch(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<BatchDeleteRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let is_admin = user
        .privs
        .get("edit_library")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let deleted = services::share::delete_batch(&state.db, &payload.ids, user.id, is_admin).await?;

    Ok(Json(serde_json::json!({"deleted": deleted})))
}

