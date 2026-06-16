use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::middleware::auth::{AuthUser, AuthenticatedUser};
use crate::services::{auth, user};
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct AddUserRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub privs: Option<serde_json::Value>,
    pub scrobbling_enabled: Option<bool>,
    pub max_bit_rate: Option<i32>,
}

#[derive(Deserialize)]
pub struct IdRequest {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct AdminPasswordEditRequest {
    pub id: i32,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct EditPrivsRequest {
    pub id: i32,
    pub privs: serde_json::Value,
}

#[derive(Deserialize)]
pub struct AdminEditUserRequest {
    pub id: i32,
    pub email: Option<Option<String>>,
    pub scrobbling_enabled: Option<bool>,
    pub max_bit_rate: Option<i32>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct AdminUserInfo {
    pub id: i32,
    pub username: String,
    pub privs: serde_json::Value,
    pub email: Option<String>,
    pub scrobbling_enabled: bool,
    pub max_bit_rate: i32,
    pub created_at: i64,
    pub updated_at: Option<i64>,
}

#[derive(Serialize)]
pub struct ListUsersResponse {
    pub users: Vec<AdminUserInfo>,
}

// ---------------------------------------------------------------------------
// Privilege checks
// ---------------------------------------------------------------------------

fn check_edit_user(auth_user: &AuthUser) -> Result<(), AppError> {
    let has_priv = auth_user
        .privs
        .get("edit_user")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !has_priv {
        return Err(AppError::forbidden(
            "Insufficient privileges: edit_user required",
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/admin/user/add
pub async fn add_user(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<AddUserRequest>,
) -> Result<(StatusCode, Json<AdminUserInfo>), AppError> {
    check_edit_user(&user)?;

    let password_hash = auth::hash_password(&payload.password)?;

    let privs_str = payload
        .privs
        .map(|v| serde_json::to_string(&v).unwrap_or_else(|_| "{}".to_string()))
        .unwrap_or_else(|| "{}".to_string());

    let inserted = user::create(
        &state.db,
        &payload.username,
        &password_hash,
        payload.email.as_deref(),
        &privs_str,
        payload.scrobbling_enabled.unwrap_or(false),
        payload.max_bit_rate.unwrap_or(0),
    )
    .await?;

    Ok((StatusCode::CREATED, Json(AdminUserInfo {
        id: inserted.id,
        username: inserted.username,
        privs: serde_json::from_str(&inserted.privs).unwrap_or_default(),
        email: inserted.email,
        scrobbling_enabled: inserted.scrobbling_enabled != 0,
        max_bit_rate: inserted.max_bit_rate,
        created_at: inserted.created_at,
        updated_at: Some(inserted.updated_at),
    })))
}

/// POST /api/admin/user/del
pub async fn del_user(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<IdRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_edit_user(&user)?;

    if user.id == payload.id {
        return Err(AppError::validation_error("Cannot delete yourself"));
    }

    user::delete(&state.db, payload.id).await?;

    Ok(Json(serde_json::json!({"message": "User deleted"})))
}

/// POST /api/admin/user/password
pub async fn admin_password_edit(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<AdminPasswordEditRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_edit_user(&user)?;

    let new_hash = auth::hash_password(&payload.new_password)?;
    user::set_password(&state.db, payload.id, &new_hash).await?;

    Ok(Json(serde_json::json!({"message": "Password updated"})))
}

/// GET /api/admin/user/list
pub async fn list_users(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<ListUsersResponse>, AppError> {
    check_edit_user(&user)?;

    let all_users = user::list_all(&state.db).await?;

    let users: Vec<AdminUserInfo> = all_users
        .into_iter()
        .map(|m| AdminUserInfo {
            id: m.id,
            username: m.username,
            privs: serde_json::from_str(&m.privs).unwrap_or_default(),
            email: m.email,
            scrobbling_enabled: m.scrobbling_enabled != 0,
            max_bit_rate: m.max_bit_rate,
            created_at: m.created_at,
            updated_at: Some(m.updated_at),
        })
        .collect();

    Ok(Json(ListUsersResponse { users }))
}

/// POST /api/admin/user/info
pub async fn user_info(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<IdRequest>,
) -> Result<Json<AdminUserInfo>, AppError> {
    check_edit_user(&user)?;

    let target = user::find_by_id(&state.db, payload.id).await?;

    Ok(Json(AdminUserInfo {
        id: target.id,
        username: target.username,
        privs: serde_json::from_str(&target.privs).unwrap_or_default(),
        email: target.email,
        scrobbling_enabled: target.scrobbling_enabled != 0,
        max_bit_rate: target.max_bit_rate,
        created_at: target.created_at,
        updated_at: Some(target.updated_at),
    }))
}

/// POST /api/admin/user/privs
pub async fn edit_privs(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<EditPrivsRequest>,
) -> Result<Json<AdminUserInfo>, AppError> {
    check_edit_user(&user)?;

    let privs_str = serde_json::to_string(&payload.privs)
        .map_err(|e| AppError::validation_error(format!("Invalid privs JSON: {e}")))?;

    let updated = user::set_privs(&state.db, payload.id, &privs_str).await?;

    Ok(Json(AdminUserInfo {
        id: updated.id,
        username: updated.username,
        privs: serde_json::from_str(&updated.privs).unwrap_or_default(),
        email: updated.email,
        scrobbling_enabled: updated.scrobbling_enabled != 0,
        max_bit_rate: updated.max_bit_rate,
        created_at: updated.created_at,
        updated_at: Some(updated.updated_at),
    }))
}

/// PUT /api/admin/user/edit
pub async fn edit_user(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<AdminEditUserRequest>,
) -> Result<Json<AdminUserInfo>, AppError> {
    check_edit_user(&user)?;

    let updated = user::admin_update(
        &state.db,
        payload.id,
        payload.email,
        payload.scrobbling_enabled,
        payload.max_bit_rate,
    )
    .await?;

    Ok(Json(AdminUserInfo {
        id: updated.id,
        username: updated.username,
        privs: serde_json::from_str(&updated.privs).unwrap_or_default(),
        email: updated.email,
        scrobbling_enabled: updated.scrobbling_enabled != 0,
        max_bit_rate: updated.max_bit_rate,
        created_at: updated.created_at,
        updated_at: Some(updated.updated_at),
    }))
}

/// POST /api/admin/user/enable
pub async fn enable_user(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<IdRequest>,
) -> Result<Json<AdminUserInfo>, AppError> {
    check_edit_user(&user)?;

    user::set_enabled(&state.db, payload.id, true).await?;
    let target = user::find_by_id(&state.db, payload.id).await?;

    Ok(Json(AdminUserInfo {
        id: target.id,
        username: target.username,
        privs: serde_json::from_str(&target.privs).unwrap_or_default(),
        email: target.email,
        scrobbling_enabled: target.scrobbling_enabled != 0,
        max_bit_rate: target.max_bit_rate,
        created_at: target.created_at,
        updated_at: Some(target.updated_at),
    }))
}

/// POST /api/admin/user/disable
pub async fn disable_user(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<IdRequest>,
) -> Result<Json<AdminUserInfo>, AppError> {
    check_edit_user(&user)?;

    if user.id == payload.id {
        return Err(AppError::validation_error("Cannot disable yourself"));
    }

    user::set_enabled(&state.db, payload.id, false).await?;
    let target = user::find_by_id(&state.db, payload.id).await?;

    Ok(Json(AdminUserInfo {
        id: target.id,
        username: target.username,
        privs: serde_json::from_str(&target.privs).unwrap_or_default(),
        email: target.email,
        scrobbling_enabled: target.scrobbling_enabled != 0,
        max_bit_rate: target.max_bit_rate,
        created_at: target.created_at,
        updated_at: Some(target.updated_at),
    }))
}
