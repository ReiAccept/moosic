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
pub struct SearchRequest {
    pub query: String,
    #[serde(rename = "type")]
    pub r#type: Option<String>,
    pub year_from: Option<i32>,
    pub year_to: Option<i32>,
    pub offset: Option<i32>,
    pub limit: Option<i32>,
}

#[derive(Deserialize)]
pub struct SuggestRequest {
    pub query: String,
    pub limit: Option<i32>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct SearchResult {
    pub artists: Vec<ArtistHit>,
    pub albums: Vec<AlbumHit>,
    pub songs: Vec<SongHit>,
    #[serde(rename = "artist_total")]
    pub total_artists: i64,
    #[serde(rename = "album_total")]
    pub total_albums: i64,
    #[serde(rename = "song_total")]
    pub total_songs: i64,
}

#[derive(Serialize)]
pub struct ArtistHit {
    pub id: i32,
    pub name: String,
    pub album_count: i64,
}

#[derive(Serialize)]
pub struct AlbumHit {
    pub id: i32,
    pub name: String,
    pub artist_name: String,
    pub year: Option<i32>,
}

#[derive(Serialize)]
pub struct SongHit {
    pub id: i32,
    pub title: String,
    pub artist_name: String,
    pub album_name: Option<String>,
    pub duration_secs: i32,
}

#[derive(Serialize)]
pub struct Suggestion {
    pub r#type: String,
    pub id: i32,
    pub text: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/search
pub async fn search(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<SearchRequest>,
) -> Result<Json<SearchResult>, AppError> {
    let _ = user;
    let query = &payload.query;
    let type_filter = payload.r#type.as_deref();
    let offset = payload.offset.unwrap_or(0).max(0) as u64;
    let limit = payload.limit.unwrap_or(50).max(1).min(500) as u64;

    let results = services::search::search(
        &state.db,
        query,
        type_filter,
        payload.year_from,
        payload.year_to,
        offset,
        limit,
    )
    .await?;

    Ok(Json(SearchResult {
        artists: results
            .artists
            .into_iter()
            .map(|a| ArtistHit {
                id: a.id,
                name: a.name,
                album_count: a.album_count,
            })
            .collect(),
        albums: results
            .albums
            .into_iter()
            .map(|a| AlbumHit {
                id: a.id,
                name: a.name,
                artist_name: a.artist_name,
                year: a.year,
            })
            .collect(),
        songs: results
            .songs
            .into_iter()
            .map(|s| SongHit {
                id: s.id,
                title: s.title,
                artist_name: s.artist_name,
                album_name: s.album_name,
                duration_secs: s.duration_secs,
            })
            .collect(),
        total_artists: results.artist_total as i64,
        total_albums: results.album_total as i64,
        total_songs: results.song_total as i64,
    }))
}

/// POST /api/search/suggest
pub async fn search_suggest(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<SuggestRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _ = user;
    let query = &payload.query;
    let limit = payload.limit.unwrap_or(10).max(1).min(50) as u64;

    let suggestions = services::search::suggest(&state.db, query, limit).await?;

    Ok(Json(serde_json::json!({"suggestions": suggestions})))
}
