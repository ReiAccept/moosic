use axum::Json;

use crate::utils::now_ms;

/// GET /api/health
pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": now_ms(),
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
