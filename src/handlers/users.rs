use axum::{http::StatusCode, Json};
use sea_orm::{ActiveValue, EntityTrait};
use serde::{Deserialize, Serialize};

use crate::entities::prelude::*;
use crate::state::AppState;

pub async fn create_user(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), (StatusCode, String)> {
    let username = payload.username;

    let user = UserActiveModel {
        username: ActiveValue::Set(username.clone()),
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
