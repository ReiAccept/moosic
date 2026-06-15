use std::time::Duration;

use axum::{http::StatusCode, Json};
use sea_orm::{ConnectionTrait, Statement};
use serde::{Deserialize, Serialize};
use memory_stats::memory_stats;

use crate::state::AppState;

/// Cache key for the server status response.
const STATUS_CACHE_KEY: &str = "admin:server:status";
/// Cache TTL for the status endpoint.
const STATUS_CACHE_TTL: Duration = Duration::from_secs(10);

/// `GET /api/admin/server/status` — server overview.
///
/// Results are cached for 10 seconds.
pub async fn server_status(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<ServerStatus>, (StatusCode, String)> {
    // Serve from cache when available
    if state.cache.exists(STATUS_CACHE_KEY).await {
        if let Some(cached) = state.cache.get(STATUS_CACHE_KEY).await {
            if let Ok(status) = serde_json::from_str::<ServerStatus>(&cached) {
                return Ok(Json(status));
            }
        }
    }

    let cache_backend = state.cache.kind().to_owned();

    // Lightweight connectivity check
    let db_connected = state
        .db
        .execute_raw(Statement::from_string(
            state.db.get_database_backend(),
            "SELECT 1",
        ))
        .await
        .is_ok();

    let Some(usage) = memory_stats() else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to get memory usage".to_string(),
        ));
    };

    let status = ServerStatus {
        version: env!("CARGO_PKG_VERSION").to_owned(),
        system: SystemStatus {
            memory_usage: usage.physical_mem as usize,
        },
        database: DatabaseStatus {
            backend: state.db_backend.to_owned(),
            connected: db_connected,
        },
        cache: CacheStatus {
            backend: cache_backend,
        },
        server: ServerInfo {
            host: state.server_host.clone(),
            port: state.server_port,
        },
    };

    // Populate the cache (best-effort — a failure here is harmless)
    if let Ok(json) = serde_json::to_string(&status) {
        state.cache.set(STATUS_CACHE_KEY, &json, Some(STATUS_CACHE_TTL)).await;
    }

    Ok(Json(status))
}

#[derive(Serialize, Deserialize)]
pub struct ServerStatus {
    /// Application version from Cargo.toml.
    version: String,
    system: SystemStatus,
    database: DatabaseStatus,
    cache: CacheStatus,
    server: ServerInfo,
}

#[derive(Serialize, Deserialize)]
pub struct SystemStatus {
    /// Physical memory usage in bytes.
    memory_usage: usize,
}

#[derive(Serialize, Deserialize)]
pub struct DatabaseStatus {
    /// Database backend name, e.g. `"sqlite"`.
    backend: String,
    /// Whether the database accepted a lightweight ping query.
    connected: bool,
}

#[derive(Serialize, Deserialize)]
pub struct CacheStatus {
    /// Cache backend name: `"memory"` or `"redis"`.
    backend: String,
}

#[derive(Serialize, Deserialize)]
pub struct ServerInfo {
    /// Listen host.
    host: String,
    /// Listen port.
    port: u16,
}
