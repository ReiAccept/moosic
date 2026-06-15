use axum::{http::StatusCode, Json};
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};

use crate::entities::prelude::*;

pub async fn create_user(
    axum::extract::State(db): axum::extract::State<DatabaseConnection>,
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), (StatusCode, String)> {
    let username = payload.username;

    let user = UserActiveModel {
        username: ActiveValue::Set(username.clone()),
        ..Default::default()
    };

    let res = UserEntity::insert(user).exec(&db).await.map_err(|e| {
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
