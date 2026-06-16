use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::entities::prelude::*;
use crate::error::AppError;
use crate::middleware::auth::AuthenticatedUser;
use crate::services::library;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct IdRequest {
    pub id: i32,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct LibraryEntry {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub is_enabled: bool,
    pub user_enabled: bool,
    pub song_count: i64,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/library/list
pub async fn library_list(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, AppError> {
    // Get per-user library data and song counts from service
    let statuses = library::list_for_user(&state.db, user.id).await?;

    // Get library-level enabled status
    let all_libraries = LibraryEntity::find().all(&state.db).await?;
    let lib_enabled_map: std::collections::HashMap<i32, bool> = all_libraries
        .into_iter()
        .map(|l| (l.id, l.is_enabled != 0))
        .collect();

    let result: Vec<LibraryEntry> = statuses
        .into_iter()
        .map(|s| LibraryEntry {
            id: s.id,
            name: s.name,
            path: s.path,
            is_enabled: lib_enabled_map.get(&s.id).copied().unwrap_or(false),
            user_enabled: s.is_enabled,
            song_count: s.song_count,
        })
        .collect();

    Ok(Json(serde_json::json!({"libraries": result})))
}

/// POST /api/library/enable
pub async fn library_enable(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<IdRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    library::enable_for_user(&state.db, user.id, payload.id).await?;
    Ok(Json(serde_json::json!({"status": "enabled"})))
}

/// POST /api/library/disable
pub async fn library_disable(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<IdRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    library::disable_for_user(&state.db, user.id, payload.id).await?;
    Ok(Json(serde_json::json!({"status": "disabled"})))
}

/// POST /api/library/rescan
pub async fn library_rescan(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(payload): Json<IdRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _ = state;
    let _ = payload;
    Ok(Json(serde_json::json!({
        "message": "Scan started",
        "scan_id": "stub"
    })))
}

