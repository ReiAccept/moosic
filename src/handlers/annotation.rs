use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::middleware::auth::AuthenticatedUser;
use crate::services;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct StarRequest {
    #[serde(rename = "type")]
    pub r#type: String,
    pub id: i32,
}

#[derive(Deserialize)]
pub struct RateRequest {
    #[serde(rename = "type")]
    pub r#type: String,
    pub id: i32,
    pub rating: i32,
}

#[derive(Deserialize)]
pub struct ClearRateRequest {
    #[serde(rename = "type")]
    pub r#type: String,
    pub id: i32,
}

#[derive(Deserialize)]
pub struct ScrobbleRequest {
    pub song_id: i32,
    pub submission: Option<bool>,
    pub played_at: Option<i64>,
}

#[derive(Deserialize)]
pub struct PagedRequest {
    pub offset: Option<i32>,
    pub limit: Option<i32>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct StarredList {
    pub artists: Vec<StarredItem>,
    pub albums: Vec<StarredItem>,
    pub songs: Vec<StarredItem>,
    pub artist_total: u64,
    pub album_total: u64,
    pub song_total: u64,
}

#[derive(Serialize)]
pub struct StarredItem {
    pub id: i32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_name: Option<String>,
    pub starred_at: i64,
}

#[derive(Serialize)]
pub struct RatedList {
    pub songs: Vec<RatedItem>,
    pub albums: Vec<RatedItem>,
    pub song_total: u64,
    pub album_total: u64,
}

#[derive(Serialize)]
pub struct RatedItem {
    pub id: i32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist_name: Option<String>,
    pub rating: i32,
    pub rated_at: i64,
}

#[derive(Serialize)]
pub struct ScrobbleEntry {
    pub id: i32,
    pub song_id: i32,
    pub title: String,
    pub artist_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_name: Option<String>,
    pub played_at: i64,
}

#[derive(Serialize)]
pub struct PlayHistory {
    pub entries: Vec<ScrobbleEntry>,
    pub total: u64,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/star
pub async fn toggle_star(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<StarRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let starred = services::annotation::toggle_star(
        &state.db,
        user.id,
        &payload.r#type,
        payload.id,
    )
    .await?;

    Ok(Json(serde_json::json!({"starred": starred})))
}

/// POST /api/rate
pub async fn set_rating(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<RateRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rating = services::annotation::set_rating(
        &state.db,
        user.id,
        &payload.r#type,
        payload.id,
        payload.rating,
    )
    .await?;

    Ok(Json(serde_json::json!({"rating": rating})))
}

/// POST /api/rate/clear
pub async fn clear_rating(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<ClearRateRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    services::annotation::clear_rating(
        &state.db,
        user.id,
        &payload.r#type,
        payload.id,
    )
    .await?;

    Ok(Json(serde_json::json!({"rating": null})))
}

/// POST /api/scrobble
pub async fn scrobble(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<ScrobbleRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let submission = payload.submission.unwrap_or(false);
    let played_at = payload.played_at.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    });

    services::annotation::scrobble(
        &state.db,
        user.id,
        payload.song_id,
        submission,
        played_at,
    )
    .await?;

    Ok(Json(serde_json::json!({"status": "ok"})))
}

/// POST /api/starred/list
pub async fn starred_list(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<PagedRequest>,
) -> Result<Json<StarredList>, AppError> {
    let offset = payload.offset.unwrap_or(0).max(0) as u64;
    let limit = payload.limit.unwrap_or(50).max(1).min(500) as u64;

    let results = services::annotation::get_starred(&state.db, user.id, offset, limit).await?;

    Ok(Json(StarredList {
        artists: results.artists.into_iter().map(|i| StarredItem {
            id: i.id,
            name: i.name,
            artist_name: i.artist_name,
            album_name: i.album_name,
            starred_at: i.starred_at,
        }).collect(),
        albums: results.albums.into_iter().map(|i| StarredItem {
            id: i.id,
            name: i.name,
            artist_name: i.artist_name,
            album_name: i.album_name,
            starred_at: i.starred_at,
        }).collect(),
        songs: results.songs.into_iter().map(|i| StarredItem {
            id: i.id,
            name: i.name,
            artist_name: i.artist_name,
            album_name: i.album_name,
            starred_at: i.starred_at,
        }).collect(),
        artist_total: results.artist_total,
        album_total: results.album_total,
        song_total: results.song_total,
    }))
}

/// POST /api/rated/list
pub async fn rated_list(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<PagedRequest>,
) -> Result<Json<RatedList>, AppError> {
    let offset = payload.offset.unwrap_or(0).max(0) as u64;
    let limit = payload.limit.unwrap_or(50).max(1).min(500) as u64;

    let results = services::annotation::get_rated(&state.db, user.id, offset, limit).await?;

    Ok(Json(RatedList {
        songs: results.songs.into_iter().map(|i| RatedItem {
            id: i.id,
            name: i.name,
            artist_name: i.artist_name,
            rating: i.rating,
            rated_at: i.rated_at,
        }).collect(),
        albums: results.albums.into_iter().map(|i| RatedItem {
            id: i.id,
            name: i.name,
            artist_name: i.artist_name,
            rating: i.rating,
            rated_at: i.rated_at,
        }).collect(),
        song_total: results.song_total,
        album_total: results.album_total,
    }))
}

/// POST /api/scrobble/history
pub async fn play_history(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<PagedRequest>,
) -> Result<Json<PlayHistory>, AppError> {
    let offset = payload.offset.unwrap_or(0).max(0) as u64;
    let limit = payload.limit.unwrap_or(50).max(1).min(500) as u64;

    let results = services::annotation::get_history(&state.db, user.id, offset, limit).await?;

    Ok(Json(PlayHistory {
        entries: results.entries.into_iter().map(|e| ScrobbleEntry {
            id: e.id,
            song_id: e.song_id,
            title: e.title,
            artist_name: e.artist_name,
            album_name: e.album_name,
            played_at: e.played_at,
        }).collect(),
        total: results.total,
    }))
}
