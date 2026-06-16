use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::entities::libraries;
use crate::error::AppError;
use crate::middleware::auth::{AuthUser, AuthenticatedUser};
use crate::services::{library, scanner};
use crate::state::{AppState, ScanStatus};

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct AddLibraryRequest {
    pub name: String,
    pub path: String,
}

#[derive(Deserialize)]
pub struct IdRequest {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct UpdateLibraryRequest {
    pub id: i32,
    pub name: Option<String>,
    pub path: Option<String>,
}

#[derive(Deserialize)]
pub struct CancelScanRequest {
    pub scan_id: String,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct LibraryInfo {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub is_enabled: bool,
    pub watch_enabled: bool,
    pub created_at: i64,
}

#[derive(Serialize)]
pub struct DeleteLibraryResponse {
    pub deleted: bool,
    pub songs_count: u64,
    pub albums_count: u64,
    pub artists_count: u64,
}

// ---------------------------------------------------------------------------
// Model helpers
// ---------------------------------------------------------------------------

fn model_to_library_info(m: libraries::Model) -> LibraryInfo {
    LibraryInfo {
        id: m.id,
        name: m.name,
        path: m.path,
        is_enabled: m.is_enabled != 0,
        watch_enabled: m.watch_enabled != 0,
        created_at: m.created_at,
    }
}

fn check_edit_library(auth_user: &AuthUser) -> Result<(), AppError> {
    let has_priv = auth_user
        .privs
        .get("edit_library")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !has_priv {
        return Err(AppError::forbidden(
            "Insufficient privileges: edit_library required",
        ));
    }
    Ok(())
}

fn scan_status_to_str(s: &ScanStatus) -> &'static str {
    match s {
        ScanStatus::Scanning => "scanning",
        ScanStatus::Completed => "completed",
        ScanStatus::Failed => "failed",
        ScanStatus::Cancelled => "cancelled",
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/admin/library/add
pub async fn add_library(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<AddLibraryRequest>,
) -> Result<(StatusCode, Json<LibraryInfo>), AppError> {
    check_edit_library(&user)?;

    let inserted = library::add_library(&state.db, &payload.name, &payload.path).await?;

    Ok((StatusCode::CREATED, Json(model_to_library_info(inserted))))
}

/// POST /api/admin/library/del
pub async fn del_library(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<IdRequest>,
) -> Result<Json<DeleteLibraryResponse>, AppError> {
    check_edit_library(&user)?;

    let counts = library::delete_library(&state.db, payload.id).await?;

    Ok(Json(DeleteLibraryResponse {
        deleted: true,
        songs_count: counts.songs,
        albums_count: counts.albums,
        artists_count: counts.artists,
    }))
}

/// PUT /api/admin/library/update
pub async fn update_library(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<UpdateLibraryRequest>,
) -> Result<Json<LibraryInfo>, AppError> {
    check_edit_library(&user)?;

    let updated = library::update_library(
        &state.db,
        payload.id,
        payload.name,
        payload.path,
    )
    .await?;

    Ok(Json(model_to_library_info(updated)))
}

/// POST /api/admin/library/enable-notify
pub async fn enable_notify(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<IdRequest>,
) -> Result<Json<LibraryInfo>, AppError> {
    check_edit_library(&user)?;

    let lib_id = payload.id;

    // Reject if the library is currently being scanned
    {
        let scan_state = state.scan_state.read().await;
        if let Some(progress) = &scan_state.active {
            if progress.library_ids.contains(&lib_id) && progress.status == ScanStatus::Scanning
            {
                return Err(AppError::conflict(
                    "Library is currently being scanned",
                ));
            }
        }
    }

    library::enable_notify(&state.db, lib_id).await?;
    let target = library::get_by_id(&state.db, lib_id).await?;

    Ok(Json(model_to_library_info(target)))
}

/// POST /api/admin/library/disable-notify
pub async fn disable_notify(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<IdRequest>,
) -> Result<Json<LibraryInfo>, AppError> {
    check_edit_library(&user)?;

    library::disable_notify(&state.db, payload.id).await?;
    let target = library::get_by_id(&state.db, payload.id).await?;

    Ok(Json(model_to_library_info(target)))
}

/// GET /api/admin/library/scan-status
pub async fn scan_status(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, AppError> {
    check_edit_library(&user)?;

    match scanner::get_status(&state.scan_state).await {
        Some(progress) => Ok(Json(serde_json::json!({
            "status": scan_status_to_str(&progress.status),
            "scan_id": progress.scan_id,
            "files_scanned": progress.files_scanned,
            "files_total": progress.files_total,
            "started_at": progress.started_at,
            "error": progress.error,
        }))),
        None => Ok(Json(serde_json::json!({"status": "idle"}))),
    }
}

/// POST /api/admin/library/cancel-scan
pub async fn cancel_scan(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CancelScanRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_edit_library(&user)?;

    scanner::cancel_scan(&state.scan_state, &payload.scan_id).await?;

    Ok(Json(serde_json::json!({"message": "Scan cancelled"})))
}

/// POST /api/admin/library/scan
pub async fn start_scan(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<ScanRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_edit_library(&user)?;

    let scan_id = scanner::start_scan(state.db.clone(), state.scan_state.clone(), payload.library_ids).await?;

    Ok(Json(serde_json::json!({
        "message": "Scan started",
        "scan_id": scan_id,
    })))
}

/// POST /api/admin/library/scan/all
pub async fn start_full_scan(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, AppError> {
    check_edit_library(&user)?;

    let scan_id = scanner::start_scan(state.db.clone(), state.scan_state.clone(), vec![]).await?;

    Ok(Json(serde_json::json!({
        "message": "Full scan started",
        "scan_id": scan_id,
    })))
}

#[derive(Deserialize)]
pub struct ScanRequest {
    pub library_ids: Vec<i32>,
}
