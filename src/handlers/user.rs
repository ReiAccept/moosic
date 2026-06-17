use axum::{extract::State, http::HeaderMap, Json};
use sea_orm::{EntityTrait};
use serde::{Deserialize, Serialize};

use crate::entities::{users};
use crate::entities::prelude::*;
use crate::error::AppError;
use crate::middleware::auth::AuthenticatedUser;
use crate::services::{auth, user};
use crate::state::AppState;


fn extract_bearer_token(headers: &HeaderMap) -> Result<&str, AppError> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::unauthorized("Missing Authorization header"))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::unauthorized("Invalid Authorization header format"))?;

    Ok(token)
}

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Serialize)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub privs: serde_json::Value,
    pub email: Option<String>,
    pub scrobbling_enabled: bool,
    pub max_bit_rate: i32,
    pub created_at: i64,
    pub updated_at: Option<i64>,
}

#[derive(Deserialize)]
pub struct PasswordEditRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct EditRequest {
    pub email: Option<Option<String>>,
    pub scrobbling_enabled: Option<bool>,
    pub max_bit_rate: Option<i32>,
}

#[derive(Deserialize)]
pub struct RevokeRequest {
    pub session_id: String,
}

#[derive(Deserialize)]
pub struct DeleteAccountRequest {
    pub password: String,
}

// ---------------------------------------------------------------------------
// Model helpers
// ---------------------------------------------------------------------------

fn model_to_user_info(model: users::Model) -> UserInfo {
    UserInfo {
        id: model.id,
        username: model.username,
        privs: serde_json::from_str(&model.privs).unwrap_or_default(),
        email: model.email,
        scrobbling_enabled: model.scrobbling_enabled != 0,
        max_bit_rate: model.max_bit_rate,
        created_at: model.created_at,
        updated_at: Some(model.updated_at),
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/auth/login
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let (token, user_model) = auth::login(&state.db, &payload.username, &payload.password).await?;
    Ok(Json(LoginResponse {
        token,
        user: model_to_user_info(user_model),
    }))
}

/// POST /api/auth/logout
pub async fn logout(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    let token = extract_bearer_token(&headers)?;
    auth::logout(&state.db, token).await?;
    Ok(Json(serde_json::json!({"message": "Logged out"})))
}

/// POST /api/auth/token/refresh
pub async fn token_refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    let token = extract_bearer_token(&headers)?;
    let new_token = auth::refresh_token(&state.db, token).await?;
    Ok(Json(serde_json::json!({"token": new_token})))
}

/// GET /api/user
pub async fn info(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<UserInfo>, AppError> {
    let model = user::find_by_id(&state.db, user.id).await?;

    Ok(Json(model_to_user_info(model)))
}

/// PUT /api/user
pub async fn edit(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<EditRequest>,
) -> Result<Json<UserInfo>, AppError> {
    let updated = user::update_profile(
        &state.db,
        user.id,
        payload.email,
        payload.scrobbling_enabled,
        payload.max_bit_rate,
    )
    .await?;

    Ok(Json(model_to_user_info(updated)))
}

/// POST /api/user/password
pub async fn password_edit(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<PasswordEditRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let model = user::find_by_id(&state.db, user.id).await?;

    let valid = auth::verify_password(&payload.old_password, &model.password_hash)?;
    if !valid {
        return Err(AppError::unauthorized("Invalid password"));
    }

    let new_hash = auth::hash_password(&payload.new_password)?;

    user::set_password(&state.db, user.id, &new_hash).await?;

    Ok(Json(serde_json::json!({"message": "Password updated"})))
}

/// GET /api/user/sessions
pub async fn sessions(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_sessions = auth::list_sessions(&state.db, user.id).await?;
    Ok(Json(serde_json::json!({"sessions": user_sessions})))
}

/// DELETE /api/user/sessions
pub async fn session_revoke(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<RevokeRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let session = SessionEntity::find_by_id(payload.session_id.clone())
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("Session not found"))?;

    if session.user_id != user.id {
        return Err(AppError::forbidden("Session does not belong to user"));
    }

    auth::revoke_session(&state.db, &payload.session_id, user.id).await?;

    Ok(Json(serde_json::json!({"message": "Session revoked"})))
}

/// DELETE /api/user
pub async fn delete_account(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<DeleteAccountRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let model = user::find_by_id(&state.db, user.id).await?;

    let valid = auth::verify_password(&payload.password, &model.password_hash)?;
    if !valid {
        return Err(AppError::unauthorized("Invalid password"));
    }

    user::delete(&state.db, user.id).await?;

    Ok(Json(serde_json::json!({"message": "Account deleted"})))
}
