use std::ops::Deref;

use axum::body::Body;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;

use crate::entities::prelude::*;
use crate::entities::sessions;
use crate::error::AppError;
use crate::state::AppState;
use crate::utils::now_ms;

/// Authenticated user information extracted from a valid session token.
#[derive(Clone, Debug)]
pub struct AuthUser {
    pub id: i32,
    pub username: String,
    pub privs: serde_json::Value,
    pub scrobbling_enabled: bool,
    pub max_bit_rate: i32,
}

/// Extractor that pulls an `AuthUser` from request extensions.
///
/// Only available on routes protected by the `auth_middleware` layer.
/// Returns `401 Unauthorized` if no authenticated user is present.
#[derive(Clone, Debug)]
pub struct AuthenticatedUser(pub AuthUser);

impl Deref for AuthenticatedUser {
    type Target = AuthUser;

    fn deref(&self) -> &AuthUser {
        &self.0
    }
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .map(AuthenticatedUser)
            .ok_or_else(|| AppError::unauthorized("Not authenticated"))
    }
}


/// Returns `true` when the request path does not require authentication.
fn is_public_path(path: &str) -> bool {
    path == "/"
        || path.starts_with("/api/health")
        || path.starts_with("/api/share/")
        || path.starts_with("/api/user/login")
        || path.starts_with("/api/user/password/reset")
}

/// Middleware that validates `Authorization: Bearer <token>` headers and
/// injects an `AuthUser` into request extensions for authenticated routes.
pub async fn auth_middleware(
    state: axum::extract::State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let path = req.uri().path();

    // Allow unauthenticated access to public paths.
    if is_public_path(path) {
        return Ok(next.run(req).await);
    }

    // Extract the Bearer token from the Authorization header.
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    let token = match token {
        Some(t) => t,
        None => return Err(AppError::unauthorized("Missing or invalid Authorization header")),
    };

    // Look up the session by token.
    let session = SessionEntity::find()
        .filter(sessions::Column::Token.eq(&token))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::unauthorized("Invalid session token"))?;

    // Look up the associated user.
    let user = UserEntity::find_by_id(session.user_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::unauthorized("User not found"))?;

    // Reject expired sessions.
    let now = now_ms();
    if session.expires_at < now {
        return Err(AppError::unauthorized("Session expired"));
    }

    // Reject disabled accounts.
    if user.is_enabled == 0 {
        return Err(AppError::unauthorized("User account is disabled"));
    }

    // Touch the session's last_used_at timestamp.
    let mut session_active: sessions::ActiveModel = session.into();
    session_active.last_used_at = ActiveValue::Set(now);
    session_active.update(&state.db).await?;

    // Build the authenticated-user payload and inject it into the request.
    let auth_user = AuthUser {
        id: user.id,
        username: user.username,
        privs: serde_json::from_str(&user.privs).unwrap_or(serde_json::Value::Null),
        scrobbling_enabled: user.scrobbling_enabled != 0,
        max_bit_rate: user.max_bit_rate,
    };

    req.extensions_mut().insert(auth_user);
    Ok(next.run(req).await)
}
