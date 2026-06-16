use std::time::{SystemTime, UNIX_EPOCH};

use axum::{extract::{Query, State}, http::StatusCode, Json};
use sea_orm::{ColumnTrait, EntityTrait, JoinType, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Order};
use serde::{Deserialize, Serialize};

use crate::entities::prelude::*;
use crate::entities::{albums, artists, songs, scrobbles, stars};
use crate::error::AppError;
use crate::middleware::auth::AuthenticatedUser;
use crate::state::AppState;

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct IdRequest {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct ArtistListRequest {
    pub letter: Option<String>,
    pub offset: Option<i32>,
    pub limit: Option<i32>,
}

#[derive(Deserialize)]
pub struct CoverQuery {
    pub id: i32,
    pub size: Option<i32>,
}

#[derive(Deserialize)]
pub struct ArtistSongsRequest {
    pub id: i32,
    pub offset: Option<i32>,
    pub limit: Option<i32>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct AlbumSummary {
    id: i32,
    name: String,
    year: Option<i32>,
    song_count: i64,
    duration_secs: i64,
    starred: Option<i64>,
}

#[derive(Serialize)]
pub struct ArtistInfoResponse {
    id: i32,
    name: String,
    sort_name: Option<String>,
    album_count: i64,
    song_count: i64,
    play_count: i64,
    starred: Option<i64>,
    albums: Vec<AlbumSummary>,
}

#[derive(Serialize)]
pub struct ArtistListItem {
    id: i32,
    name: String,
    sort_name: Option<String>,
    album_count: i64,
    song_count: i64,
    cover_url: String,
    starred: Option<i64>,
}

#[derive(Serialize)]
pub struct SongListItem {
    id: i32,
    title: String,
    artist_id: i32,
    album_id: Option<i32>,
    track_number: Option<i32>,
    disc_number: Option<i32>,
    duration_secs: i32,
    bit_rate: Option<i32>,
    size_bytes: Option<i64>,
    file_format: Option<String>,
    content_type: Option<String>,
    year: Option<i32>,
    has_cover_art: i32,
    starred: Option<i64>,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub total: i64,
    pub items: Vec<T>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/artist/info
pub async fn artist_info(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(params): Json<IdRequest>,
) -> Result<Json<ArtistInfoResponse>, AppError> {
    let artist = ArtistEntity::find_by_id(params.id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("Artist not found"))?;

    // Count albums
    let album_count = AlbumEntity::find()
        .filter(albums::Column::ArtistId.eq(params.id))
        .count(&state.db)
        .await?;

    // Count songs
    let song_count = SongEntity::find()
        .filter(songs::Column::ArtistId.eq(params.id))
        .count(&state.db)
        .await?;

    // Count plays: scrobbles WHERE submission = 1 AND song.artist_id = artist.id
    let play_count = ScrobbleEntity::find()
        .join(JoinType::InnerJoin, scrobbles::Relation::Song.def())
        .filter(songs::Column::ArtistId.eq(params.id))
        .filter(scrobbles::Column::Submission.eq(1))
        .count(&state.db)
        .await?;

    // Starred status for the artist
    let starred = StarEntity::find()
        .filter(stars::Column::UserId.eq(user.id))
        .filter(stars::Column::ItemType.eq("artist"))
        .filter(stars::Column::ItemId.eq(params.id))
        .one(&state.db)
        .await?;

    // Albums belonging to this artist
    let album_models = AlbumEntity::find()
        .filter(albums::Column::ArtistId.eq(params.id))
        .order_by(albums::Column::Year, Order::Desc)
        .all(&state.db)
        .await?;

    let mut albums_out = Vec::with_capacity(album_models.len());
    for album in album_models {
        let songs_in_album = SongEntity::find()
            .filter(songs::Column::AlbumId.eq(album.id))
            .all(&state.db)
            .await?;

        let a_song_count = songs_in_album.len() as i64;
        let a_duration_secs: i64 = songs_in_album.iter().map(|s| s.duration_secs as i64).sum();

        let album_starred = StarEntity::find()
            .filter(stars::Column::UserId.eq(user.id))
            .filter(stars::Column::ItemType.eq("album"))
            .filter(stars::Column::ItemId.eq(album.id))
            .one(&state.db)
            .await?;

        albums_out.push(AlbumSummary {
            id: album.id,
            name: album.name,
            year: album.year,
            song_count: a_song_count,
            duration_secs: a_duration_secs,
            starred: album_starred.map(|s| s.starred_at),
        });
    }

    Ok(Json(ArtistInfoResponse {
        id: artist.id,
        name: artist.name,
        sort_name: artist.sort_name,
        album_count: album_count as i64,
        song_count: song_count as i64,
        play_count: play_count as i64,
        starred: starred.map(|s| s.starred_at),
        albums: albums_out,
    }))
}

/// POST /api/artist/list
pub async fn artist_list(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(params): Json<ArtistListRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = (params.limit.unwrap_or(50).max(1).min(500)) as u64;
    let offset = (params.offset.unwrap_or(0).max(0)) as u64;

    let mut base_query = ArtistEntity::find();

    if let Some(ref letter) = params.letter {
        if !letter.is_empty() {
            base_query = base_query.filter(artists::Column::Name.like(format!("{}%", letter)));
        }
    }

    let total = base_query.clone().count(&state.db).await? as i64;

    let artist_models = base_query
        .order_by(artists::Column::Name, Order::Asc)
        .limit(limit)
        .offset(offset)
        .all(&state.db)
        .await?;

    let mut items = Vec::with_capacity(artist_models.len());
    for artist in artist_models {
        let album_count = AlbumEntity::find()
            .filter(albums::Column::ArtistId.eq(artist.id))
            .count(&state.db)
            .await?;

        let song_count = SongEntity::find()
            .filter(songs::Column::ArtistId.eq(artist.id))
            .count(&state.db)
            .await?;

        let starred = StarEntity::find()
            .filter(stars::Column::UserId.eq(user.id))
            .filter(stars::Column::ItemType.eq("artist"))
            .filter(stars::Column::ItemId.eq(artist.id))
            .one(&state.db)
            .await?;

        items.push(ArtistListItem {
            id: artist.id,
            name: artist.name,
            sort_name: artist.sort_name,
            album_count: album_count as i64,
            song_count: song_count as i64,
            cover_url: format!("/api/artist/cover?id={}", artist.id),
            starred: starred.map(|s| s.starred_at),
        });
    }

    Ok(Json(serde_json::json!({"artists": items, "total": total})))
}

/// GET /api/artist/cover?id=<id>
pub async fn artist_cover(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Query(params): Query<CoverQuery>,
) -> Result<(StatusCode, [(String, String); 2], Vec<u8>), AppError> {
    let (mime_type, data) = crate::services::cover::get_cover_response(&state.db, "artist", params.id).await?;
    Ok((
        StatusCode::OK,
        [
            ("Content-Type".into(), mime_type),
            ("Cache-Control".into(), "public, max-age=604800".into()),
        ],
        data,
    ))
}

/// POST /api/artist/songs
pub async fn artist_songs(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(params): Json<ArtistSongsRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = (params.limit.unwrap_or(50).max(1).min(500)) as u64;
    let offset = (params.offset.unwrap_or(0).max(0)) as u64;

    let total = SongEntity::find()
        .filter(songs::Column::ArtistId.eq(params.id))
        .count(&state.db)
        .await? as i64;

    let song_models = SongEntity::find()
        .filter(songs::Column::ArtistId.eq(params.id))
        .order_by(songs::Column::DiscNumber, Order::Asc)
        .order_by(songs::Column::TrackNumber, Order::Asc)
        .limit(limit)
        .offset(offset)
        .all(&state.db)
        .await?;

    let mut items = Vec::with_capacity(song_models.len());
    for song in song_models {
        let starred = StarEntity::find()
            .filter(stars::Column::UserId.eq(user.id))
            .filter(stars::Column::ItemType.eq("song"))
            .filter(stars::Column::ItemId.eq(song.id))
            .one(&state.db)
            .await?;

        items.push(SongListItem {
            id: song.id,
            title: song.title,
            artist_id: song.artist_id,
            album_id: song.album_id,
            track_number: song.track_number,
            disc_number: song.disc_number,
            duration_secs: song.duration_secs,
            bit_rate: song.bit_rate,
            size_bytes: song.size_bytes,
            file_format: song.file_format,
            content_type: song.content_type,
            year: song.year,
            has_cover_art: song.has_cover_art,
            starred: starred.map(|s| s.starred_at),
        });
    }

    Ok(Json(serde_json::json!({"songs": items, "total": total})))
}
