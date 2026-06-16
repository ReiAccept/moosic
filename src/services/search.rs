use std::collections::HashMap;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use serde::Serialize;

use crate::entities::prelude::*;
use crate::entities::{albums, artists, songs};
use crate::error::AppError;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Aggregated search results across artists, albums and songs.
#[derive(Serialize)]
pub struct SearchResults {
    pub artists: Vec<ArtistResult>,
    pub albums: Vec<AlbumResult>,
    pub songs: Vec<SongResult>,
    pub artist_total: u64,
    pub album_total: u64,
    pub song_total: u64,
}

#[derive(Serialize)]
pub struct ArtistResult {
    pub id: i32,
    pub name: String,
    pub album_count: i64,
}

#[derive(Serialize)]
pub struct AlbumResult {
    pub id: i32,
    pub name: String,
    pub artist_name: String,
    pub year: Option<i32>,
    pub song_count: i64,
}

#[derive(Serialize)]
pub struct SongResult {
    pub id: i32,
    pub title: String,
    pub artist_name: String,
    pub album_name: Option<String>,
    pub track_number: Option<i32>,
    pub duration_secs: i32,
}

/// A single suggestion for the auto-complete endpoint.
#[derive(Serialize)]
pub struct Suggestion {
    #[serde(rename = "type")]
    pub r#type: String,
    pub id: i32,
    pub text: String,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Full-text search with LIKE '%query%' across artists, albums and songs.
///
/// When `type_filter` is `None` or `"all"` every category is searched;
/// otherwise only the matching category (`"artist"`, `"album"`, `"song"`) is queried.
/// Year filters are applied to albums and songs.
pub async fn search(
    db: &DatabaseConnection,
    query: &str,
    type_filter: Option<&str>,
    year_from: Option<i32>,
    year_to: Option<i32>,
    offset: u64,
    limit: u64,
) -> Result<SearchResults, AppError> {
    let search_all = type_filter.map_or(true, |t| t == "all");
    let like = format!("%{}%", query);

    let mut result = SearchResults {
        artists: Vec::new(),
        albums: Vec::new(),
        songs: Vec::new(),
        artist_total: 0,
        album_total: 0,
        song_total: 0,
    };

    // ---- Artists -----------------------------------------------------------
    if search_all || type_filter == Some("artist") {
        let artist_query =
            ArtistEntity::find().filter(artists::Column::Name.like(&like));

        result.artist_total = artist_query.clone().count(db).await?;
        let artist_models = artist_query.offset(offset).limit(limit).all(db).await?;

        // Count albums per artist.
        let artist_ids: Vec<i32> = artist_models.iter().map(|a| a.id).collect();
        let album_counts: HashMap<i32, i64> = if !artist_ids.is_empty() {
            AlbumEntity::find()
                .filter(albums::Column::ArtistId.is_in(artist_ids))
                .all(db)
                .await?
                .into_iter()
                .fold(HashMap::new(), |mut acc, a| {
                    *acc.entry(a.artist_id).or_insert(0) += 1;
                    acc
                })
        } else {
            HashMap::new()
        };

        result.artists = artist_models
            .into_iter()
            .map(|a| ArtistResult {
                id: a.id,
                name: a.name,
                album_count: album_counts.get(&a.id).copied().unwrap_or(0),
            })
            .collect();
    }

    // ---- Albums ------------------------------------------------------------
    if search_all || type_filter == Some("album") {
        let mut album_query =
            AlbumEntity::find().filter(albums::Column::Name.like(&like));

        if let Some(y) = year_from {
            album_query = album_query.filter(albums::Column::Year.gte(y));
        }
        if let Some(y) = year_to {
            album_query = album_query.filter(albums::Column::Year.lte(y));
        }

        result.album_total = album_query.clone().count(db).await?;
        let album_models = album_query.offset(offset).limit(limit).all(db).await?;

        // Resolve artist names and count songs.
        let artist_ids: Vec<i32> = album_models.iter().map(|a| a.artist_id).collect();
        let album_ids: Vec<i32> = album_models.iter().map(|a| a.id).collect();

        let artist_names: HashMap<i32, String> = if !artist_ids.is_empty() {
            ArtistEntity::find()
                .filter(artists::Column::Id.is_in(artist_ids))
                .all(db)
                .await?
                .into_iter()
                .map(|a| (a.id, a.name))
                .collect()
        } else {
            HashMap::new()
        };

        let song_counts: HashMap<i32, i64> = if !album_ids.is_empty() {
            SongEntity::find()
                .filter(songs::Column::AlbumId.is_in(album_ids))
                .all(db)
                .await?
                .into_iter()
                .fold(HashMap::new(), |mut acc, s| {
                    if let Some(aid) = s.album_id {
                        *acc.entry(aid).or_insert(0) += 1;
                    }
                    acc
                })
        } else {
            HashMap::new()
        };

        result.albums = album_models
            .into_iter()
            .map(|a| AlbumResult {
                id: a.id,
                name: a.name,
                artist_name: artist_names
                    .get(&a.artist_id)
                    .cloned()
                    .unwrap_or_default(),
                year: a.year,
                song_count: song_counts.get(&a.id).copied().unwrap_or(0),
            })
            .collect();
    }

    // ---- Songs -------------------------------------------------------------
    if search_all || type_filter == Some("song") {
        let mut song_query =
            SongEntity::find().filter(songs::Column::Title.like(&like));

        if let Some(y) = year_from {
            song_query = song_query.filter(songs::Column::Year.gte(y));
        }
        if let Some(y) = year_to {
            song_query = song_query.filter(songs::Column::Year.lte(y));
        }

        result.song_total = song_query.clone().count(db).await?;
        let song_models = song_query.offset(offset).limit(limit).all(db).await?;

        // Resolve artist and album names.
        let artist_ids: Vec<i32> = song_models.iter().map(|s| s.artist_id).collect();
        let album_ids: Vec<i32> = song_models.iter().filter_map(|s| s.album_id).collect();

        let artist_names: HashMap<i32, String> = ArtistEntity::find()
            .filter(artists::Column::Id.is_in(artist_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|a| (a.id, a.name))
            .collect();

        let album_names: HashMap<i32, String> = if !album_ids.is_empty() {
            AlbumEntity::find()
                .filter(albums::Column::Id.is_in(album_ids))
                .all(db)
                .await?
                .into_iter()
                .map(|a| (a.id, a.name))
                .collect()
        } else {
            HashMap::new()
        };

        result.songs = song_models
            .into_iter()
            .map(|s| SongResult {
                id: s.id,
                title: s.title,
                artist_name: artist_names
                    .get(&s.artist_id)
                    .cloned()
                    .unwrap_or_default(),
                album_name: s
                    .album_id
                    .and_then(|aid| album_names.get(&aid).cloned()),
                track_number: s.track_number,
                duration_secs: s.duration_secs,
            })
            .collect();
    }

    Ok(result)
}

/// Prefix auto-complete (`LIKE 'query%'`) across artists, albums and songs.
///
/// Album suggestions are formatted as `"Artist -- Album"`, song suggestions as
/// `"Artist -- Title"`.  The total number of results is capped by `limit`.
pub async fn suggest(
    db: &DatabaseConnection,
    query: &str,
    limit: u64,
) -> Result<Vec<Suggestion>, AppError> {
    let mut suggestions: Vec<Suggestion> = Vec::new();

    if query.is_empty() {
        return Ok(suggestions);
    }

    let prefix = format!("{}%", query);

    // ---- Artists (prefix) --------------------------------------------------
    let artist_models = ArtistEntity::find()
        .filter(artists::Column::Name.like(&prefix))
        .limit(limit)
        .all(db)
        .await?;

    for a in &artist_models {
        if suggestions.len() >= limit as usize {
            break;
        }
        suggestions.push(Suggestion {
            r#type: "artist".to_string(),
            id: a.id,
            text: a.name.clone(),
        });
    }

    // ---- Albums (prefix) ---------------------------------------------------
    let album_models = AlbumEntity::find()
        .filter(albums::Column::Name.like(&prefix))
        .limit(limit)
        .all(db)
        .await?;

    let album_artist_ids: Vec<i32> = album_models.iter().map(|a| a.artist_id).collect();
    let album_artist_names: HashMap<i32, String> = if !album_artist_ids.is_empty() {
        ArtistEntity::find()
            .filter(artists::Column::Id.is_in(album_artist_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|a| (a.id, a.name))
            .collect()
    } else {
        HashMap::new()
    };

    for a in &album_models {
        if suggestions.len() >= limit as usize {
            break;
        }
        let artist = album_artist_names
            .get(&a.artist_id)
            .map(|s| s.as_str())
            .unwrap_or("Unknown");
        suggestions.push(Suggestion {
            r#type: "album".to_string(),
            id: a.id,
            text: format!("{} -- {}", artist, a.name),
        });
    }

    // ---- Songs (prefix) ----------------------------------------------------
    let song_models = SongEntity::find()
        .filter(songs::Column::Title.like(&prefix))
        .limit(limit)
        .all(db)
        .await?;

    let song_artist_ids: Vec<i32> = song_models.iter().map(|s| s.artist_id).collect();
    let song_artist_names: HashMap<i32, String> = if !song_artist_ids.is_empty() {
        ArtistEntity::find()
            .filter(artists::Column::Id.is_in(song_artist_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|a| (a.id, a.name))
            .collect()
    } else {
        HashMap::new()
    };

    for s in &song_models {
        if suggestions.len() >= limit as usize {
            break;
        }
        let artist = song_artist_names
            .get(&s.artist_id)
            .map(|n| n.as_str())
            .unwrap_or("Unknown");
        suggestions.push(Suggestion {
            r#type: "song".to_string(),
            id: s.id,
            text: format!("{} -- {}", artist, s.title),
        });
    }

    Ok(suggestions)
}
