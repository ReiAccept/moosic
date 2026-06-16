use axum::{http::StatusCode, Json};
use sea_orm::{ActiveValue, EntityTrait};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::entities::prelude::*;
use crate::state::AppState;

/// Current time as 13-digit Unix milliseconds.
fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub async fn create_user(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), (StatusCode, String)> {
    let username = payload.username;
    let now = now_ms();

    let user = UserActiveModel {
        username: ActiveValue::Set(username.clone()),
        created_at: ActiveValue::Set(now),
        ..Default::default()
    };

    let res = UserEntity::insert(user).exec(&state.db).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to insert user: {e}"),
        )
    })?;

    Ok((
        StatusCode::CREATED,
        Json(User {
            id: res.last_insert_id,
            username,
        }),
    ))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
pub struct User {
    pub id: i32,
    pub username: String,
}
