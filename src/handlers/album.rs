use std::time::{SystemTime, UNIX_EPOCH};

use axum::{extract::{Query, State}, http::StatusCode, Json};
use sea_orm::{
    ColumnTrait, EntityTrait, JoinType, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Order, Statement,
};
use serde::{Deserialize, Serialize};

use crate::entities::prelude::*;
use crate::entities::{albums, artists, songs, stars};
use crate::error::AppError;
use crate::middleware::auth::AuthenticatedUser;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct IdRequest {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct AlbumListRequest {
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
pub struct SongSummary {
    id: i32,
    title: String,
    artist_id: i32,
    album_id: Option<i32>,
    track_number: Option<i32>,
    disc_number: Option<i32>,
    duration_secs: i32,
    bit_rate: Option<i32>,
    year: Option<i32>,
    has_cover_art: i32,
    starred: Option<i64>,
}

#[derive(Serialize)]
pub struct AlbumInfoResponse {
    id: i32,
    name: String,
    artist_id: i32,
    artist_name: String,
    year: Option<i32>,
    song_count: i64,
    duration_secs: i64,
    created_at: i64,
    starred: Option<i64>,
    songs: Vec<SongSummary>,
}

#[derive(Serialize)]
pub struct AlbumListItem {
    id: i32,
    name: String,
    artist_id: i32,
    artist_name: String,
    year: Option<i32>,
    song_count: i64,
    duration_secs: i64,
    created_at: i64,
    starred: Option<i64>,
    cover_url: String,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    total: i64,
    items: Vec<T>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a single `AlbumListItem` from an album model by fetching its
/// artist name, song count, duration, and star status.
async fn album_to_list_item(
    db: &sea_orm::DatabaseConnection,
    user_id: i32,
    album: albums::Model,
) -> Result<AlbumListItem, AppError> {
    let artist = ArtistEntity::find_by_id(album.artist_id)
        .one(db)
        .await?
        .map(|a| a.name)
        .unwrap_or_default();

    let songs_in_album = SongEntity::find()
        .filter(songs::Column::AlbumId.eq(album.id))
        .all(db)
        .await?;

    let song_count = songs_in_album.len() as i64;
    let duration_secs: i64 = songs_in_album.iter().map(|s| s.duration_secs as i64).sum();

    let starred = StarEntity::find()
        .filter(stars::Column::UserId.eq(user_id))
        .filter(stars::Column::ItemType.eq("album"))
        .filter(stars::Column::ItemId.eq(album.id))
        .one(db)
        .await?;

    Ok(AlbumListItem {
        id: album.id,
        name: album.name,
        artist_id: album.artist_id,
        artist_name: artist,
        year: album.year,
        song_count,
        duration_secs,
        created_at: album.created_at,
        starred: starred.map(|s| s.starred_at),
        cover_url: format!("/api/album/cover?id={}", album.id),
    })
}

/// Execute a raw SQL SELECT that returns album IDs, then fetch the full
/// album models in the same order.
async fn fetch_albums_by_ids_raw(
    state: &AppState,
    sql: &str,
    limit: u64,
    offset: u64,
) -> Result<Vec<albums::Model>, AppError> {
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
        if let Some(model) = AlbumEntity::find_by_id(id).one(&state.db).await? {
            ordered.push(model);
        }
    }

    Ok(ordered)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/album/info
pub async fn album_info(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(params): Json<IdRequest>,
) -> Result<Json<AlbumInfoResponse>, AppError> {
    let album = AlbumEntity::find_by_id(params.id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("Album not found"))?;

    let artist = ArtistEntity::find_by_id(album.artist_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("Artist not found"))?;

    let song_models = SongEntity::find()
        .filter(songs::Column::AlbumId.eq(params.id))
        .order_by(songs::Column::DiscNumber, Order::Asc)
        .order_by(songs::Column::TrackNumber, Order::Asc)
        .all(&state.db)
        .await?;

    let song_count = song_models.len() as i64;
    let duration_secs: i64 = song_models.iter().map(|s| s.duration_secs as i64).sum();

    // Album starred status
    let starred = StarEntity::find()
        .filter(stars::Column::UserId.eq(user.id))
        .filter(stars::Column::ItemType.eq("album"))
        .filter(stars::Column::ItemId.eq(params.id))
        .one(&state.db)
        .await?;

    // Per-song starred status
    let mut songs_out = Vec::with_capacity(song_models.len());
    for s in song_models {
        let song_starred = StarEntity::find()
            .filter(stars::Column::UserId.eq(user.id))
            .filter(stars::Column::ItemType.eq("song"))
            .filter(stars::Column::ItemId.eq(s.id))
            .one(&state.db)
            .await?;

        songs_out.push(SongSummary {
            id: s.id,
            title: s.title,
            artist_id: s.artist_id,
            album_id: s.album_id,
            track_number: s.track_number,
            disc_number: s.disc_number,
            duration_secs: s.duration_secs,
            bit_rate: s.bit_rate,
            year: s.year,
            has_cover_art: s.has_cover_art,
            starred: song_starred.map(|st| st.starred_at),
        });
    }

    Ok(Json(AlbumInfoResponse {
        id: album.id,
        name: album.name,
        artist_id: album.artist_id,
        artist_name: artist.name,
        year: album.year,
        song_count,
        duration_secs,
        created_at: album.created_at,
        starred: starred.map(|s| s.starred_at),
        songs: songs_out,
    }))
}

/// POST /api/album/list
pub async fn album_list(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(params): Json<AlbumListRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = (params.limit.unwrap_or(50).max(1).min(500)) as u64;
    let offset = (params.offset.unwrap_or(0).max(0)) as u64;

    let items: Vec<AlbumListItem> = match params.r#type.as_str() {
        "newest" => {
            let mut query = AlbumEntity::find()
                .order_by(albums::Column::CreatedAt, Order::Desc);

            if let Some(aid) = params.artist_id {
                query = query.filter(albums::Column::ArtistId.eq(aid));
            }
            if let Some(yf) = params.year_from {
                query = query.filter(albums::Column::Year.gte(yf));
            }
            if let Some(yt) = params.year_to {
                query = query.filter(albums::Column::Year.lte(yt));
            }

            let models = query.limit(limit).offset(offset).all(&state.db).await?;
            let mut out = Vec::with_capacity(models.len());
            for m in models {
                out.push(album_to_list_item(&state.db, user.id, m).await?);
            }
            out
        }
        "recent" => {
            // Most recently played albums (via scrobbles)
            let mut sql = String::from(
                "SELECT a.id FROM albums a \
                 INNER JOIN songs s ON s.album_id = a.id \
                 INNER JOIN scrobbles sc ON sc.song_id = s.id \
                 WHERE sc.submission = 1",
            );
            if let Some(aid) = params.artist_id {
                sql.push_str(&format!(" AND a.artist_id = {}", aid));
            }
            sql.push_str(" GROUP BY a.id ORDER BY MAX(sc.played_at) DESC");

            let models = fetch_albums_by_ids_raw(&state, &sql, limit, offset).await?;
            let mut out = Vec::with_capacity(models.len());
            for m in models {
                out.push(album_to_list_item(&state.db, user.id, m).await?);
            }
            out
        }
        "frequent" => {
            // Most played albums (by play count)
            let mut sql = String::from(
                "SELECT a.id FROM albums a \
                 INNER JOIN songs s ON s.album_id = a.id \
                 INNER JOIN scrobbles sc ON sc.song_id = s.id \
                 WHERE sc.submission = 1",
            );
            if let Some(aid) = params.artist_id {
                sql.push_str(&format!(" AND a.artist_id = {}", aid));
            }
            sql.push_str(" GROUP BY a.id ORDER BY COUNT(sc.id) DESC");

            let models = fetch_albums_by_ids_raw(&state, &sql, limit, offset).await?;
            let mut out = Vec::with_capacity(models.len());
            for m in models {
                out.push(album_to_list_item(&state.db, user.id, m).await?);
            }
            out
        }
        "random" => {
            let mut sql = String::from("SELECT id FROM albums");
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

            let models = fetch_albums_by_ids_raw(&state, &sql, limit, offset).await?;
            let mut out = Vec::with_capacity(models.len());
            for m in models {
                out.push(album_to_list_item(&state.db, user.id, m).await?);
            }
            out
        }
        "alphabeticalByName" => {
            let mut query = AlbumEntity::find()
                .order_by(albums::Column::Name, Order::Asc);

            if let Some(aid) = params.artist_id {
                query = query.filter(albums::Column::ArtistId.eq(aid));
            }
            if let Some(yf) = params.year_from {
                query = query.filter(albums::Column::Year.gte(yf));
            }
            if let Some(yt) = params.year_to {
                query = query.filter(albums::Column::Year.lte(yt));
            }

            let models = query.limit(limit).offset(offset).all(&state.db).await?;
            let mut out = Vec::with_capacity(models.len());
            for m in models {
                out.push(album_to_list_item(&state.db, user.id, m).await?);
            }
            out
        }
        "alphabeticalByArtist" => {
            let mut query = AlbumEntity::find()
                .join(JoinType::InnerJoin, albums::Relation::Artists.def())
                .order_by(artists::Column::Name, Order::Asc)
                .order_by(albums::Column::Name, Order::Asc);

            if let Some(aid) = params.artist_id {
                query = query.filter(albums::Column::ArtistId.eq(aid));
            }
            if let Some(yf) = params.year_from {
                query = query.filter(albums::Column::Year.gte(yf));
            }
            if let Some(yt) = params.year_to {
                query = query.filter(albums::Column::Year.lte(yt));
            }

            let models = query.limit(limit).offset(offset).all(&state.db).await?;
            let mut out = Vec::with_capacity(models.len());
            for m in models {
                out.push(album_to_list_item(&state.db, user.id, m).await?);
            }
            out
        }
        "starred" => {
            // Fetch album IDs starred by the current user
            let starred_entries = StarEntity::find()
                .filter(stars::Column::UserId.eq(user.id))
                .filter(stars::Column::ItemType.eq("album"))
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

                let mut out = Vec::with_capacity(paged.len());
                for id in paged {
                    if let Some(album) = AlbumEntity::find_by_id(id).one(&state.db).await? {
                        out.push(album_to_list_item(&state.db, user.id, album).await?);
                    }
                }
                out
            }
        }
        "byYear" => {
            let from = params.year_from.unwrap_or(0);
            let to = params.year_to.unwrap_or(9999);

            let mut query = AlbumEntity::find()
                .filter(albums::Column::Year.between(from, to))
                .order_by(albums::Column::Year, Order::Desc);

            if let Some(aid) = params.artist_id {
                query = query.filter(albums::Column::ArtistId.eq(aid));
            }

            let models = query.limit(limit).offset(offset).all(&state.db).await?;
            let mut out = Vec::with_capacity(models.len());
            for m in models {
                out.push(album_to_list_item(&state.db, user.id, m).await?);
            }
            out
        }
        _ => {
            return Err(AppError::validation_error(format!(
                "Unknown album list type: {}",
                params.r#type
            )));
        }
    };

    // Compute total count
    let total = if params.r#type == "starred" {
        StarEntity::find()
            .filter(stars::Column::UserId.eq(user.id))
            .filter(stars::Column::ItemType.eq("album"))
            .count(&state.db)
            .await? as i64
    } else {
        let mut count_query = AlbumEntity::find();

        if let Some(aid) = params.artist_id {
            count_query = count_query.filter(albums::Column::ArtistId.eq(aid));
        }

        match params.r#type.as_str() {
            "byYear" => {
                let from = params.year_from.unwrap_or(0);
                let to = params.year_to.unwrap_or(9999);
                count_query = count_query.filter(albums::Column::Year.between(from, to));
            }
            _ => {
                if let Some(yf) = params.year_from {
                    count_query = count_query.filter(albums::Column::Year.gte(yf));
                }
                if let Some(yt) = params.year_to {
                    count_query = count_query.filter(albums::Column::Year.lte(yt));
                }
            }
        }

        count_query.count(&state.db).await? as i64
    };

    Ok(Json(serde_json::json!({"albums": items, "total": total})))
}

/// GET /api/album/cover?id=<id>
pub async fn album_cover(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Query(params): Query<CoverQuery>,
) -> Result<(StatusCode, [(String, String); 2], Vec<u8>), AppError> {
    let (mime_type, data) = crate::services::cover::get_cover_response(&state.db, "album", params.id).await?;
    Ok((
        StatusCode::OK,
        [
            ("Content-Type".into(), mime_type),
            ("Cache-Control".into(), "public, max-age=604800".into()),
        ],
        data,
    ))
}
