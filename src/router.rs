use axum::{routing::{get, post}, Router};
use sea_orm::DatabaseConnection;

use crate::handlers;

pub fn create_router(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/", get(handlers::root))
        .route("/users", post(handlers::create_user))
        .with_state(db)
}
