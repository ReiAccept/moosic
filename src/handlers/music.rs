use std::time::{SystemTime, UNIX_EPOCH};

use axum::{extract::{Query, State}, http::{HeaderMap, StatusCode}, Json};
use sea_orm::{
    ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Order, Statement,
};
use serde::{Deserialize, Serialize};

use crate::entities::prelude::*;
use crate::entities::{songs, stars, scrobbles};
use crate::error::AppError;
use crate::middleware::auth::AuthenticatedUser;
use crate::services;
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
pub struct StreamQuery {
    pub id: i32,
    pub max_bit_rate: Option<i32>,
}

#[derive(Deserialize)]
pub struct IdOnlyQuery {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct RandRequest {
    pub size: Option<i32>,
    pub year_from: Option<i32>,
    pub year_to: Option<i32>,
}

#[derive(Deserialize)]
pub struct EmptyRequest {}

#[derive(Deserialize)]
pub struct MusicListRequest {
    pub r#type: String,
    pub artist_id: Option<i32>,
    pub year_from: Option<i32>,
    pub year_to: Option<i32>,
    pub offset: Option<i32>,
    pub limit: Option<i32>,
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
pub struct MusicInfoResponse {
    id: i32,
    title: String,
    artist_id: i32,
    artist_name: String,
    album_id: Option<i32>,
    album_name: Option<String>,
    track_number: Option<i32>,
    disc_number: Option<i32>,
    duration_secs: i32,
    bit_rate: Option<i32>,
    size_bytes: Option<i64>,
    file_format: Option<String>,
    content_type: Option<String>,
    year: Option<i32>,
    has_cover_art: i32,
    file_path: String,
    play_count: i64,
    starred: Option<i64>,
    created_at: i64,
}

#[derive(Serialize)]
pub struct SongDetail {
    id: i32,
    title: String,
    artist_id: i32,
    artist_name: Option<String>,
    album_id: Option<i32>,
    album_name: Option<String>,
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
    play_count: i64,
}

#[derive(Serialize)]
pub struct LyricLine {
    start_ms: Option<i64>,
    text: String,
}

#[derive(Serialize)]
pub struct LyricsResponse {
    r#type: String,
    lines: Vec<LyricLine>,
}

#[derive(Serialize)]
pub struct NowPlayingEntry {
    song_id: i32,
    title: String,
    artist_name: String,
    device_id: Option<String>,
    minutes_ago: i64,
    username: String,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    total: i64,
    items: Vec<T>,
}

// ---------------------------------------------------------------------------
// LRC parsing helpers
// ---------------------------------------------------------------------------

/// Parse a single LRC timestamp `[mm:ss.xx]` or `[mm:ss]` into milliseconds.
fn parse_lrc_timestamp(ts: &str) -> Option<i64> {
    let parts: Vec<&str> = ts.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }
    let minutes: i64 = parts[0].parse().ok()?;
    let sec_part = parts[1];

    if let Some((secs, centis)) = sec_part.split_once('.') {
        let seconds: i64 = secs.parse().ok()?;
        let centiseconds: i64 = centis.parse().ok()?;
        Some(minutes * 60_000 + seconds * 1000 + centiseconds * 10)
    } else {
        let seconds: i64 = sec_part.parse().ok()?;
        Some(minutes * 60_000 + seconds * 1000)
    }
}

/// Parse LRC-formatted lyrics into a list of timed lines.
fn parse_lrc(content: &str) -> Vec<LyricLine> {
    let mut lines = Vec::new();
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix('[') {
            if let Some((timestamp_str, text)) = rest.split_once(']') {
                if let Some(ms) = parse_lrc_timestamp(timestamp_str) {
                    lines.push(LyricLine {
                        start_ms: Some(ms),
                        text: text.trim().to_string(),
                    });
                    continue;
                }
            }
        }
        lines.push(LyricLine {
            start_ms: None,
            text: line.to_string(),
        });
    }
    lines
}

/// Convert plain unsynced lyrics into the line format.
fn parse_unsynced(content: &str) -> Vec<LyricLine> {
    content
        .lines()
        .map(|line| LyricLine {
            start_ms: None,
            text: line.trim().to_string(),
        })
        .filter(|l| !l.text.is_empty())
        .collect()
}

// ---------------------------------------------------------------------------
// Song detail builder (single-item)
// ---------------------------------------------------------------------------

async fn build_song_detail(
    db: &sea_orm::DatabaseConnection,
    user_id: i32,
    song: songs::Model,
) -> Result<SongDetail, AppError> {
    let artist = ArtistEntity::find_by_id(song.artist_id).one(db).await?;
    let album = if let Some(aid) = song.album_id {
        AlbumEntity::find_by_id(aid).one(db).await?
    } else {
        None
    };

    let play_count = ScrobbleEntity::find()
        .filter(scrobbles::Column::SongId.eq(song.id))
        .filter(scrobbles::Column::Submission.eq(1))
        .count(db)
        .await? as i64;

    let starred = StarEntity::find()
        .filter(stars::Column::UserId.eq(user_id))
        .filter(stars::Column::ItemType.eq("song"))
        .filter(stars::Column::ItemId.eq(song.id))
        .one(db)
        .await?;

    Ok(SongDetail {
        id: song.id,
        title: song.title,
        artist_id: song.artist_id,
        artist_name: artist.map(|a| a.name),
        album_id: song.album_id,
        album_name: album.map(|a| a.name),
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
        play_count,
    })
}

// ---------------------------------------------------------------------------
// Raw-SQL fetch helpers
// ---------------------------------------------------------------------------

/// Execute a raw SQL SELECT that returns song IDs, then fetch the full
/// song models in the same order.
async fn fetch_songs_by_ids_raw(
    state: &AppState,
    sql: &str,
    limit: u64,
    offset: u64,
) -> Result<Vec<songs::Model>, AppError> {
    let paginated_sql = format!("{} LIMIT {} OFFSET {}", sql, limit, offset);

    let rows = state
        .db
        .query_all_raw(Statement::from_string(
            state.db.get_database_backend(),
            paginated_sql,
        ))
        .await?;

    let ids: Vec<i32> = rows
        .iter()
        .filter_map(|row| row.try_get_by_index::<i32>(0).ok())
        .collect();

    let mut ordered = Vec::with_capacity(ids.len());
    for id in ids {
        if let Some(model) = SongEntity::find_by_id(id).one(&state.db).await? {
            ordered.push(model);
        }
    }

    Ok(ordered)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/music/info
pub async fn music_info(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(params): Json<IdRequest>,
) -> Result<Json<MusicInfoResponse>, AppError> {
    let song = SongEntity::find_by_id(params.id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("Song not found"))?;

    let artist = ArtistEntity::find_by_id(song.artist_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("Artist not found"))?;

    let album = if let Some(aid) = song.album_id {
        AlbumEntity::find_by_id(aid).one(&state.db).await?
    } else {
        None
    };

    let play_count = ScrobbleEntity::find()
        .filter(scrobbles::Column::SongId.eq(song.id))
        .filter(scrobbles::Column::Submission.eq(1))
        .count(&state.db)
        .await? as i64;

    let starred = StarEntity::find()
        .filter(stars::Column::UserId.eq(user.id))
        .filter(stars::Column::ItemType.eq("song"))
        .filter(stars::Column::ItemId.eq(song.id))
        .one(&state.db)
        .await?;

    Ok(Json(MusicInfoResponse {
        id: song.id,
        title: song.title,
        artist_id: song.artist_id,
        artist_name: artist.name,
        album_id: song.album_id,
        album_name: album.map(|a| a.name),
        track_number: song.track_number,
        disc_number: song.disc_number,
        duration_secs: song.duration_secs,
        bit_rate: song.bit_rate,
        size_bytes: song.size_bytes,
        file_format: song.file_format,
        content_type: song.content_type,
        year: song.year,
        has_cover_art: song.has_cover_art,
        file_path: song.file_path,
        play_count,
        starred: starred.map(|s| s.starred_at),
        created_at: song.created_at,
    }))
}

/// GET /api/music/stream?id=<id>
pub async fn music_stream(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(params): Query<StreamQuery>,
) -> Result<(StatusCode, HeaderMap, Vec<u8>), AppError> {
    let stream_info = services::media::get_stream_data(&state.db, user.id, params.id).await?;

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", stream_info.content_type.parse().unwrap());
    headers.insert("Accept-Ranges", "bytes".parse().unwrap());
    headers.insert("Cache-Control", "public, max-age=31536000".parse().unwrap());

    Ok((StatusCode::OK, headers, stream_info.data))
}

/// GET /api/music/download?id=<id>
pub async fn music_download(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(params): Query<IdOnlyQuery>,
) -> Result<(StatusCode, HeaderMap, Vec<u8>), AppError> {
    let stream_info = services::media::get_stream_data(&state.db, user.id, params.id).await?;

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", stream_info.content_type.parse().unwrap());
    headers.insert("Accept-Ranges", "bytes".parse().unwrap());
    headers.insert("Cache-Control", "public, max-age=31536000".parse().unwrap());

    if let Some(ref name) = stream_info.file_name {
        headers.insert("Content-Disposition", format!("attachment; filename=\"{}\"", name).parse().unwrap());
    }

    Ok((StatusCode::OK, headers, stream_info.data))
}

/// GET /api/music/cover?id=<id>
pub async fn music_cover(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Query(params): Query<CoverQuery>,
) -> Result<(StatusCode, [(String, String); 2], Vec<u8>), AppError> {
    let (mime_type, data) = crate::services::cover::get_cover_response(&state.db, "song", params.id).await?;
    Ok((
        StatusCode::OK,
        [
            ("Content-Type".into(), mime_type),
            ("Cache-Control".into(), "public, max-age=604800".into()),
        ],
        data,
    ))
}

/// GET /api/music/lyrics?id=<id>
pub async fn music_lyrics(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Query(params): Query<IdOnlyQuery>,
) -> Result<Json<LyricsResponse>, AppError> {
    let lyric = LyricEntity::find_by_id(params.id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("Lyrics not found"))?;

    let lines = match lyric.type_.as_str() {
        "synced" => parse_lrc(&lyric.content),
        _ => parse_unsynced(&lyric.content),
    };

    Ok(Json(LyricsResponse {
        r#type: lyric.type_,
        lines,
    }))
}

/// POST /api/music/rand
pub async fn music_rand(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(params): Json<RandRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let size = (params.size.unwrap_or(10).max(1).min(100)) as u64;

    // Use raw SQL with RANDOM() since SeaORM doesn't expose order_by_rand
    let mut sql = String::from("SELECT id FROM songs");
    let mut clauses = Vec::new();

    if let Some(yf) = params.year_from {
        clauses.push(format!("year >= {}", yf));
    }
    if let Some(yt) = params.year_to {
        clauses.push(format!("year <= {}", yt));
    }
    if !clauses.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&clauses.join(" AND "));
    }
    sql.push_str(" ORDER BY RANDOM()");

    let models = fetch_songs_by_ids_raw(&state, &sql, size, 0).await?;

    let mut items = Vec::with_capacity(models.len());
    for song in models {
        items.push(build_song_detail(&state.db, user.id, song).await?);
    }

    Ok(Json(serde_json::json!({"songs": items})))
}

/// POST /api/music/playing
pub async fn music_playing(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(_params): Json<EmptyRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let now = now_ms();

    let now_playing = ScrobbleEntity::find()
        .filter(scrobbles::Column::UserId.eq(user.id))
        .filter(scrobbles::Column::Submission.eq(0))
        .order_by(scrobbles::Column::PlayedAt, Order::Desc)
        .all(&state.db)
        .await?;

    let mut items = Vec::with_capacity(now_playing.len());
    for scrobble in now_playing {
        let song = SongEntity::find_by_id(scrobble.song_id)
            .one(&state.db)
            .await?
            .ok_or_else(|| AppError::not_found("Song not found"))?;

        let artist = ArtistEntity::find_by_id(song.artist_id)
            .one(&state.db)
            .await?;

        let minutes_ago = if scrobble.played_at > 0 {
            (now - scrobble.played_at) / 60_000
        } else {
            0
        };

        items.push(NowPlayingEntry {
            song_id: scrobble.song_id,
            title: song.title,
            artist_name: artist.map(|a| a.name).unwrap_or_default(),
            device_id: None,
            minutes_ago,
            username: user.username.clone(),
        });
    }

    Ok(Json(serde_json::json!({"entries": items})))
}

/// POST /api/music/list
pub async fn music_list(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(params): Json<MusicListRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = (params.limit.unwrap_or(50).max(1).min(500)) as u64;
    let offset = (params.offset.unwrap_or(0).max(0)) as u64;

    let items: Vec<SongDetail> = match params.r#type.as_str() {
        "newest" => {
            let mut query = SongEntity::find()
                .order_by(songs::Column::CreatedAt, Order::Desc);

            if let Some(aid) = params.artist_id {
                query = query.filter(songs::Column::ArtistId.eq(aid));
            }
            if let Some(yf) = params.year_from {
                query = query.filter(songs::Column::Year.gte(yf));
            }
            if let Some(yt) = params.year_to {
                query = query.filter(songs::Column::Year.lte(yt));
            }

            let models = query.limit(limit).offset(offset).all(&state.db).await?;
            batch_enrich_songs(&state.db, user.id, models).await?
        }
        "recent" => {
            // Most recently played songs
            let mut sql = String::from(
                "SELECT sc.song_id FROM scrobbles sc \
                 WHERE sc.submission = 1",
            );
            if let Some(aid) = params.artist_id {
                sql.push_str(&format!(
                    " AND sc.song_id IN (SELECT id FROM songs WHERE artist_id = {})",
                    aid
                ));
            }
            sql.push_str(" GROUP BY sc.song_id ORDER BY MAX(sc.played_at) DESC");

            let models = fetch_songs_by_ids_raw(&state, &sql, limit, offset).await?;
            batch_enrich_songs(&state.db, user.id, models).await?
        }
        "frequent" => {
            // Most played songs
            let mut sql = String::from(
                "SELECT sc.song_id FROM scrobbles sc \
                 WHERE sc.submission = 1",
            );
            if let Some(aid) = params.artist_id {
                sql.push_str(&format!(
                    " AND sc.song_id IN (SELECT id FROM songs WHERE artist_id = {})",
                    aid
                ));
            }
            sql.push_str(" GROUP BY sc.song_id ORDER BY COUNT(sc.id) DESC");

            let models = fetch_songs_by_ids_raw(&state, &sql, limit, offset).await?;
            batch_enrich_songs(&state.db, user.id, models).await?
        }
        "random" => {
            let mut sql = String::from("SELECT id FROM songs");
            let mut clauses = Vec::new();

            if let Some(aid) = params.artist_id {
                clauses.push(format!("artist_id = {}", aid));
            }
            if let Some(yf) = params.year_from {
                clauses.push(format!("year >= {}", yf));
            }
            if let Some(yt) = params.year_to {
                clauses.push(format!("year <= {}", yt));
            }
            if !clauses.is_empty() {
                sql.push_str(" WHERE ");
                sql.push_str(&clauses.join(" AND "));
            }
            sql.push_str(" ORDER BY RANDOM()");

            let models = fetch_songs_by_ids_raw(&state, &sql, limit, offset).await?;
            batch_enrich_songs(&state.db, user.id, models).await?
        }
        "starred" => {
            let starred_entries = StarEntity::find()
                .filter(stars::Column::UserId.eq(user.id))
                .filter(stars::Column::ItemType.eq("song"))
                .all(&state.db)
                .await?;

            if starred_entries.is_empty() {
                Vec::new()
            } else {
                let mut entries = starred_entries;
                entries.sort_by(|a, b| b.starred_at.cmp(&a.starred_at));

                let paged: Vec<i32> = entries
                    .into_iter()
                    .skip(offset as usize)
                    .take(limit as usize)
                    .map(|s| s.item_id)
                    .collect();

                let mut models = Vec::with_capacity(paged.len());
                for id in paged {
                    if let Some(song) = SongEntity::find_by_id(id).one(&state.db).await? {
                        models.push(song);
                    }
                }
                batch_enrich_songs(&state.db, user.id, models).await?
            }
        }
        "byYear" => {
            let from = params.year_from.unwrap_or(0);
            let to = params.year_to.unwrap_or(9999);

            let mut query = SongEntity::find()
                .filter(songs::Column::Year.between(from, to))
                .order_by(songs::Column::Year, Order::Desc);

            if let Some(aid) = params.artist_id {
                query = query.filter(songs::Column::ArtistId.eq(aid));
            }

            let models = query.limit(limit).offset(offset).all(&state.db).await?;
            batch_enrich_songs(&state.db, user.id, models).await?
        }
        _ => {
            return Err(AppError::validation_error(format!(
                "Unknown music list type: {}",
                params.r#type
            )));
        }
    };

    let total = if params.r#type == "starred" {
        StarEntity::find()
            .filter(stars::Column::UserId.eq(user.id))
            .filter(stars::Column::ItemType.eq("song"))
            .count(&state.db)
            .await? as i64
    } else {
        let mut count_query = SongEntity::find();

        if let Some(aid) = params.artist_id {
            count_query = count_query.filter(songs::Column::ArtistId.eq(aid));
        }
        if let Some(yf) = params.year_from {
            count_query = count_query.filter(songs::Column::Year.gte(yf));
        }
        if let Some(yt) = params.year_to {
            count_query = count_query.filter(songs::Column::Year.lte(yt));
        }

        count_query.count(&state.db).await? as i64
    };

    Ok(Json(serde_json::json!({"songs": items, "total": total})))
}

// ---------------------------------------------------------------------------
// Batch enrichment
// ---------------------------------------------------------------------------

/// Enrich a collection of song models into `SongDetail` values, fetching
/// artist/album names, play counts, and star status per item.
async fn batch_enrich_songs(
    db: &sea_orm::DatabaseConnection,
    user_id: i32,
    song_models: Vec<songs::Model>,
) -> Result<Vec<SongDetail>, AppError> {
    let mut items = Vec::with_capacity(song_models.len());
    for s in song_models {
        let artist = ArtistEntity::find_by_id(s.artist_id).one(db).await?;
        let album = if let Some(aid) = s.album_id {
            AlbumEntity::find_by_id(aid).one(db).await?
        } else {
            None
        };

        let play_count = ScrobbleEntity::find()
            .filter(scrobbles::Column::SongId.eq(s.id))
            .filter(scrobbles::Column::Submission.eq(1))
            .count(db)
            .await? as i64;

        let starred = StarEntity::find()
            .filter(stars::Column::UserId.eq(user_id))
            .filter(stars::Column::ItemType.eq("song"))
            .filter(stars::Column::ItemId.eq(s.id))
            .one(db)
            .await?;

        items.push(SongDetail {
            id: s.id,
            title: s.title,
            artist_id: s.artist_id,
            artist_name: artist.map(|a| a.name),
            album_id: s.album_id,
            album_name: album.map(|a| a.name),
            track_number: s.track_number,
            disc_number: s.disc_number,
            duration_secs: s.duration_secs,
            bit_rate: s.bit_rate,
            size_bytes: s.size_bytes,
            file_format: s.file_format,
            content_type: s.content_type,
            year: s.year,
            has_cover_art: s.has_cover_art,
            starred: starred.map(|s| s.starred_at),
            play_count,
        });
    }
    Ok(items)
}
