use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::middleware::auth::AuthenticatedUser;
use crate::services;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct IdRequest {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct CreatePlaylistRequest {
    pub name: String,
    pub comment: Option<String>,
    pub is_public: Option<bool>,
    pub song_ids: Option<Vec<i32>>,
}

#[derive(Deserialize)]
pub struct UpdatePlaylistRequest {
    pub id: i32,
    pub name: Option<String>,
    pub comment: Option<Option<String>>,
    pub is_public: Option<bool>,
}

#[derive(Deserialize)]
pub struct ModifySongsRequest {
    pub id: i32,
    pub song_ids: Vec<i32>,

}

#[derive(Deserialize)]
pub struct ReorderRequest {
    pub id: i32,
    pub song_ids: Vec<i32>,
}

#[derive(Deserialize)]
pub struct CoverQuery {
    pub id: i32,
    pub size: Option<i32>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct PlaylistSummary {
    pub id: i32,
    pub name: String,
    pub owner_name: String,
    pub is_public: bool,
    pub song_count: i64,
    pub duration_secs: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Serialize)]
pub struct PlaylistDetail {
    pub id: i32,
    pub name: String,
    pub owner_id: i32,
    pub comment: Option<String>,
    pub is_public: bool,
    pub songs: Vec<PlaylistSongEntry>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Serialize)]
pub struct PlaylistSongEntry {
    pub position: i32,
    pub song_id: i32,
    pub title: String,
    pub artist_name: String,
    pub album_name: Option<String>,
    pub duration_secs: i32,
    pub starred: Option<i64>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/playlists
pub async fn playlist_list(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = services::playlist::list_visible(&state.db, user.id).await?;

    Ok(Json(serde_json::json!({"playlists": result})))
}

/// POST /api/playlist/info
pub async fn playlist_info(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<IdRequest>,
) -> Result<Json<PlaylistDetail>, AppError> {
    let (playlist, items) = services::playlist::get_detail(&state.db, payload.id, user.id).await?;

    let songs: Vec<PlaylistSongEntry> = items
        .into_iter()
        .map(|i| PlaylistSongEntry {
            position: i.position,
            song_id: i.song_id,
            title: i.title,
            artist_name: i.artist_name,
            album_name: i.album_name,
            duration_secs: i.duration_secs,
            starred: i.starred,
        })
        .collect();

    Ok(Json(PlaylistDetail {
        id: playlist.id,
        name: playlist.name,
        owner_id: playlist.owner_id,
        comment: playlist.comment,
        is_public: playlist.is_public != 0,
        songs,
        created_at: playlist.created_at,
        updated_at: playlist.updated_at,
    }))
}

/// POST /api/playlist/create
pub async fn playlist_create(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreatePlaylistRequest>,
) -> Result<(StatusCode, Json<PlaylistDetail>), AppError> {
    let song_ids: &[i32] = payload.song_ids.as_deref().unwrap_or(&[]);

    let playlist = services::playlist::create(
        &state.db,
        user.id,
        &payload.name,
        payload.comment.as_deref(),
        payload.is_public.unwrap_or(false),
        song_ids,
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(PlaylistDetail {
            id: playlist.id,
            name: playlist.name,
            owner_id: playlist.owner_id,
            comment: playlist.comment,
            is_public: playlist.is_public != 0,
            songs: Vec::new(),
            created_at: playlist.created_at,
            updated_at: playlist.updated_at,
        }),
    ))
}

/// POST /api/playlist/update
pub async fn playlist_update(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<UpdatePlaylistRequest>,
) -> Result<Json<PlaylistDetail>, AppError> {
    let updated = services::playlist::update(
        &state.db,
        payload.id,
        user.id,
        payload.name,
        payload.comment,
        payload.is_public,
    )
    .await?;

    Ok(Json(PlaylistDetail {
        id: updated.id,
        name: updated.name,
        owner_id: updated.owner_id,
        comment: updated.comment,
        is_public: updated.is_public != 0,
        songs: Vec::new(),
        created_at: updated.created_at,
        updated_at: updated.updated_at,
    }))
}

/// POST /api/playlist/delete
pub async fn playlist_del(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<IdRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    services::playlist::delete(&state.db, payload.id, user.id).await?;

    Ok(Json(serde_json::json!({"message": "Playlist deleted"})))
}

/// POST /api/playlist/add-songs
pub async fn playlist_add_songs(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<ModifySongsRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    services::playlist::add_songs(&state.db, payload.id, user.id, &payload.song_ids).await?;

    Ok(Json(serde_json::json!({"message": "Songs added"})))
}

/// POST /api/playlist/remove-songs
pub async fn playlist_remove_songs(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<ModifySongsRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    services::playlist::remove_songs(&state.db, payload.id, user.id, &payload.song_ids).await?;

    Ok(Json(serde_json::json!({"message": "Songs removed"})))
}

/// POST /api/playlist/reorder
pub async fn playlist_reorder(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<ReorderRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    services::playlist::reorder(&state.db, payload.id, user.id, &payload.song_ids).await?;

    Ok(Json(serde_json::json!({"message": "Playlist reordered"})))
}

/// GET /api/playlist/cover — placeholder SVG
pub async fn playlist_cover(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Query(_query): Query<CoverQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let svg = "\
        <svg xmlns='http://www.w3.org/2000/svg' width='300' height='300'>\
        <rect width='300' height='300' fill='#2a2a2a'/>\
        <text x='150' y='150' text-anchor='middle' dominant-baseline='central'\
        fill='#888' font-family='sans-serif' font-size='18'>No Cover</text>\
        </svg>";

    Ok(Json(serde_json::json!({
        "placeholder": true,
        "mime_type": "image/svg+xml",
        "svg": svg,
    })))
}
