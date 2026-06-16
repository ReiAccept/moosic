use argon2::password_hash::{rand_core::OsRng, PasswordHash, SaltString};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use rand::Rng;
use sea_orm::ActiveValue;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::entities::prelude::*;
use crate::entities::{sessions, users};
use crate::error::AppError;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Summary of a session returned by `list_sessions`.
#[derive(Serialize)]
pub struct SessionInfo {
    pub id: String,
    pub created_at: i64,
    pub last_used_at: i64,
    pub device_info: Option<String>,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// 13-digit Unix millisecond timestamp.
fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Token / session helpers
// ---------------------------------------------------------------------------

/// Generate a random 32-character alphanumeric token.
pub fn generate_token() -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Generate a session identifier prefixed with `sess_`.
pub fn generate_session_id() -> String {
    format!("sess_{}", generate_token())
}

// ---------------------------------------------------------------------------
// Password hashing
// ---------------------------------------------------------------------------

/// Hash a plaintext password using Argon2id.
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::internal(format!("Failed to hash password: {e}")))?
        .to_string();
    Ok(hash)
}

/// Verify a plaintext password against an Argon2id hash.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::internal(format!("Failed to parse password hash: {e}")))?;
    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

// ---------------------------------------------------------------------------
// Authentication operations
// ---------------------------------------------------------------------------

/// Authenticate a user by username and password.
///
/// Returns `(token, user_model)` on success.
pub async fn login(
    db: &DatabaseConnection,
    username: &str,
    password: &str,
) -> Result<(String, users::Model), AppError> {
    // Look up the user by username.
    let user = UserEntity::find()
        .filter(users::Column::Username.eq(username))
        .one(db)
        .await?
        .ok_or_else(|| AppError::unauthorized("Invalid username or password"))?;

    // Reject disabled accounts.
    if user.is_enabled == 0 {
        return Err(AppError::unauthorized("Account is disabled"));
    }

    // Verify the password.
    let valid = verify_password(password, &user.password_hash)?;
    if !valid {
        return Err(AppError::unauthorized("Invalid username or password"));
    }

    // Create a new session.
    let now = now_ms();
    let token = generate_token();

    let session = sessions::ActiveModel {
        id: ActiveValue::Set(generate_session_id()),
        user_id: ActiveValue::Set(user.id),
        token: ActiveValue::Set(token.clone()),
        device_info: ActiveValue::Set(None),
        created_at: ActiveValue::Set(now),
        last_used_at: ActiveValue::Set(now),
        expires_at: ActiveValue::Set(now + 30 * 24 * 3600 * 1000),
    };

    SessionEntity::insert(session).exec(db).await?;

    Ok((token, user))
}

/// Invalidate a session by its token (logout).
pub async fn logout(db: &DatabaseConnection, token: &str) -> Result<(), AppError> {
    let session = SessionEntity::find()
        .filter(sessions::Column::Token.eq(token))
        .one(db)
        .await?
        .ok_or_else(|| AppError::unauthorized("Invalid session"))?;

    SessionEntity::delete_by_id(session.id).exec(db).await?;
    Ok(())
}

/// Exchange an old (still-valid) token for a new one.
pub async fn refresh_token(
    db: &DatabaseConnection,
    old_token: &str,
) -> Result<String, AppError> {
    let now = now_ms();

    let session = SessionEntity::find()
        .filter(sessions::Column::Token.eq(old_token))
        .one(db)
        .await?
        .ok_or_else(|| AppError::unauthorized("Invalid session"))?;

    // Reject expired tokens.
    if session.expires_at < now {
        return Err(AppError::unauthorized("Session expired"));
    }

    // Remove the old session.
    SessionEntity::delete_by_id(session.id).exec(db).await?;

    // Create a fresh session for the same user.
    let token = generate_token();
    let new_session = sessions::ActiveModel {
        id: ActiveValue::Set(generate_session_id()),
        user_id: ActiveValue::Set(session.user_id),
        token: ActiveValue::Set(token.clone()),
        device_info: ActiveValue::Set(session.device_info.clone()),
        created_at: ActiveValue::Set(now),
        last_used_at: ActiveValue::Set(now),
        expires_at: ActiveValue::Set(now + 30 * 24 * 3600 * 1000),
    };

    SessionEntity::insert(new_session).exec(db).await?;

    Ok(token)
}

/// List all sessions belonging to a given user.
pub async fn list_sessions(
    db: &DatabaseConnection,
    user_id: i32,
) -> Result<Vec<SessionInfo>, AppError> {
    let sessions = SessionEntity::find()
        .filter(sessions::Column::UserId.eq(user_id))
        .all(db)
        .await?;

    Ok(sessions
        .into_iter()
        .map(|s| SessionInfo {
            id: s.id,
            created_at: s.created_at,
            last_used_at: s.last_used_at,
            device_info: s.device_info,
        })
        .collect())
}

/// Revoke (delete) a specific session, verifying ownership.
pub async fn revoke_session(
    db: &DatabaseConnection,
    session_id: &str,
    requesting_user_id: i32,
) -> Result<(), AppError> {
    let session = SessionEntity::find()
        .filter(sessions::Column::Id.eq(session_id))
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Session not found"))?;

    if session.user_id != requesting_user_id {
        return Err(AppError::forbidden(
            "Cannot revoke another user's session",
        ));
    }

    SessionEntity::delete_by_id(session.id).exec(db).await?;
    Ok(())
}
