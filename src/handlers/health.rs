use std::time::{SystemTime, UNIX_EPOCH};

use axum::Json;

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// GET /api/health
pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": now_ms(),
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
