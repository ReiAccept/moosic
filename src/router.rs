use axum::{routing::{get, post}, Router};

use crate::handlers;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::root))
        .route("/users", post(handlers::create_user))
        .route("/api/admin/server/status", get(handlers::server_status))
        .with_state(state)
}
