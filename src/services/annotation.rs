use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use sea_orm::{
    ActiveModelTrait, ActiveValue::*, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait,
    Order, QueryFilter, QueryOrder, QuerySelect,
};
use serde::Serialize;

use crate::entities::prelude::*;
use crate::entities::{albums, artists, ratings, scrobbles, songs, stars};
use crate::error::AppError;

// ---------------------------------------------------------------------------
// Timestamp helper
// ---------------------------------------------------------------------------

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct StarredResults {
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
pub struct RatedResults {
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
pub struct HistoryResults {
    pub entries: Vec<HistoryEntry>,
    pub total: u64,
}

#[derive(Serialize)]
pub struct HistoryEntry {
    pub id: i32,
    pub song_id: i32,
    pub title: String,
    pub artist_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_name: Option<String>,
    pub played_at: i64,
}

// ---------------------------------------------------------------------------
// User annotations
// ---------------------------------------------------------------------------

/// Star (or unstar) an item.  Returns `true` if the item is now starred.
pub async fn toggle_star(
    db: &DatabaseConnection,
    user_id: i32,
    item_type: &str,
    item_id: i32,
) -> Result<bool, AppError> {
    let existing = StarEntity::find()
        .filter(stars::Column::UserId.eq(user_id))
        .filter(stars::Column::ItemType.eq(item_type))
        .filter(stars::Column::ItemId.eq(item_id))
        .one(db)
        .await?;

    if let Some(star) = existing {
        star.delete(db).await?;
        Ok(false)
    } else {
        StarActiveModel {
            user_id: Set(user_id),
            item_type: Set(item_type.to_owned()),
            item_id: Set(item_id),
            starred_at: Set(now_ms()),
        }
        .insert(db)
        .await?;
        Ok(true)
    }
}

/// Set or update a rating (1–5) for an item.  Returns the rating value.
pub async fn set_rating(
    db: &DatabaseConnection,
    user_id: i32,
    item_type: &str,
    item_id: i32,
    rating: i32,
) -> Result<i32, AppError> {
    if !(1..=5).contains(&rating) {
        return Err(AppError::validation_error(
            "Rating must be between 1 and 5",
        ));
    }

    let existing = RatingEntity::find()
        .filter(ratings::Column::UserId.eq(user_id))
        .filter(ratings::Column::ItemType.eq(item_type))
        .filter(ratings::Column::ItemId.eq(item_id))
        .one(db)
        .await?;

    let now = now_ms();
    if let Some(r) = existing {
        let mut ac = r.into_active_model();
        ac.rating = Set(rating);
        ac.rated_at = Set(now);
        ac.update(db).await?;
    } else {
        RatingActiveModel {
            user_id: Set(user_id),
            item_type: Set(item_type.to_owned()),
            item_id: Set(item_id),
            rating: Set(rating),
            rated_at: Set(now),
        }
        .insert(db)
        .await?;
    }

    Ok(rating)
}

/// Remove any rating for an item.
pub async fn clear_rating(
    db: &DatabaseConnection,
    user_id: i32,
    item_type: &str,
    item_id: i32,
) -> Result<(), AppError> {
    RatingEntity::delete_many()
        .filter(ratings::Column::UserId.eq(user_id))
        .filter(ratings::Column::ItemType.eq(item_type))
        .filter(ratings::Column::ItemId.eq(item_id))
        .exec(db)
        .await?;
    Ok(())
}

/// Scrobble a song play.
///
/// When `submission` is true the play is counted toward listening history;
/// duplicate submissions within a 5-minute window are silently skipped.
pub async fn scrobble(
    db: &DatabaseConnection,
    user_id: i32,
    song_id: i32,
    submission: bool,
    played_at: i64,
) -> Result<(), AppError> {
    // Validate the song exists.
    let _song = SongEntity::find_by_id(song_id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Song not found"))?;

    // Dedup: same user + same song played within +/- 5 minutes.
    if submission {
        let duplicate = ScrobbleEntity::find()
            .filter(scrobbles::Column::UserId.eq(user_id))
            .filter(scrobbles::Column::SongId.eq(song_id))
            .filter(scrobbles::Column::PlayedAt.gte(played_at - 300_000))
            .filter(scrobbles::Column::PlayedAt.lte(played_at + 300_000))
            .filter(scrobbles::Column::Submission.eq(1))
            .one(db)
            .await?;

        if duplicate.is_some() {
            return Ok(());
        }
    }

    ScrobbleActiveModel {
        user_id: Set(user_id),
        song_id: Set(song_id),
        submission: Set(if submission { 1 } else { 0 }),
        played_at: Set(played_at),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Retrieval
// ---------------------------------------------------------------------------

/// Return all starred items for a user, grouped by type with resolved names.
pub async fn get_starred(
    db: &DatabaseConnection,
    user_id: i32,
    offset: u64,
    limit: u64,
) -> Result<StarredResults, AppError> {
    let artist_stars = StarEntity::find()
        .filter(stars::Column::UserId.eq(user_id))
        .filter(stars::Column::ItemType.eq("artist"))
        .offset(offset)
        .limit(limit)
        .all(db)
        .await?;

    let album_stars = StarEntity::find()
        .filter(stars::Column::UserId.eq(user_id))
        .filter(stars::Column::ItemType.eq("album"))
        .offset(offset)
        .limit(limit)
        .all(db)
        .await?;

    let song_stars = StarEntity::find()
        .filter(stars::Column::UserId.eq(user_id))
        .filter(stars::Column::ItemType.eq("song"))
        .offset(offset)
        .limit(limit)
        .all(db)
        .await?;

    let artist_total = StarEntity::find()
        .filter(stars::Column::UserId.eq(user_id))
        .filter(stars::Column::ItemType.eq("artist"))
        .count(db)
        .await?;

    let album_total = StarEntity::find()
        .filter(stars::Column::UserId.eq(user_id))
        .filter(stars::Column::ItemType.eq("album"))
        .count(db)
        .await?;

    let song_total = StarEntity::find()
        .filter(stars::Column::UserId.eq(user_id))
        .filter(stars::Column::ItemType.eq("song"))
        .count(db)
        .await?;

    // ---- Resolve names -----------------------------------------------------

    let artist_ids: Vec<i32> = artist_stars.iter().map(|s| s.item_id).collect();
    let artist_names: HashMap<i32, String> = if !artist_ids.is_empty() {
        ArtistEntity::find()
            .filter(artists::Column::Id.is_in(&artist_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|a| (a.id, a.name))
            .collect()
    } else {
        HashMap::new()
    };

    let album_ids: Vec<i32> = album_stars.iter().map(|s| s.item_id).collect();
    let album_infos: HashMap<i32, (String, String)> = if !album_ids.is_empty() {
        let albums = AlbumEntity::find()
            .filter(albums::Column::Id.is_in(&album_ids))
            .all(db)
            .await?;
        let a_ids: Vec<i32> = albums.iter().map(|a| a.artist_id).collect();
        let a_names: HashMap<i32, String> = ArtistEntity::find()
            .filter(artists::Column::Id.is_in(a_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|a| (a.id, a.name))
            .collect();
        albums
            .into_iter()
            .map(|a| {
                (
                    a.id,
                    (
                        a.name,
                        a_names.get(&a.artist_id).cloned().unwrap_or_default(),
                    ),
                )
            })
            .collect()
    } else {
        HashMap::new()
    };

    let song_ids: Vec<i32> = song_stars.iter().map(|s| s.item_id).collect();
    let song_infos: HashMap<i32, (String, String, Option<String>)> = if !song_ids.is_empty() {
        let songs = SongEntity::find()
            .filter(songs::Column::Id.is_in(&song_ids))
            .all(db)
            .await?;
        let a_ids: Vec<i32> = songs.iter().map(|s| s.artist_id).collect();
        let al_ids: Vec<i32> = songs.iter().filter_map(|s| s.album_id).collect();
        let a_names: HashMap<i32, String> = ArtistEntity::find()
            .filter(artists::Column::Id.is_in(a_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|a| (a.id, a.name))
            .collect();
        let al_names: HashMap<i32, String> = if !al_ids.is_empty() {
            AlbumEntity::find()
                .filter(albums::Column::Id.is_in(al_ids))
                .all(db)
                .await?
                .into_iter()
                .map(|a| (a.id, a.name))
                .collect()
        } else {
            HashMap::new()
        };
        songs
            .into_iter()
            .map(|s| {
                (
                    s.id,
                    (
                        s.title,
                        a_names.get(&s.artist_id).cloned().unwrap_or_default(),
                        s.album_id.and_then(|aid| al_names.get(&aid).cloned()),
                    ),
                )
            })
            .collect()
    } else {
        HashMap::new()
    };

    // ---- Build response ----------------------------------------------------

    Ok(StarredResults {
        artists: artist_stars
            .into_iter()
            .map(|s| StarredItem {
                id: s.item_id,
                name: artist_names
                    .get(&s.item_id)
                    .cloned()
                    .unwrap_or_default(),
                artist_name: None,
                album_name: None,
                starred_at: s.starred_at,
            })
            .collect(),
        albums: album_stars
            .into_iter()
            .map(|s| {
                let (name, artist_name) = album_infos
                    .get(&s.item_id)
                    .cloned()
                    .unwrap_or_default();
                StarredItem {
                    id: s.item_id,
                    name,
                    artist_name: Some(artist_name),
                    album_name: None,
                    starred_at: s.starred_at,
                }
            })
            .collect(),
        songs: song_stars
            .into_iter()
            .map(|s| {
                let (title, artist_name, album_name) = song_infos
                    .get(&s.item_id)
                    .cloned()
                    .unwrap_or_default();
                StarredItem {
                    id: s.item_id,
                    name: title,
                    artist_name: Some(artist_name),
                    album_name,
                    starred_at: s.starred_at,
                }
            })
            .collect(),
        artist_total,
        album_total,
        song_total,
    })
}

/// Return all rated items for a user, grouped by type with resolved names.
pub async fn get_rated(
    db: &DatabaseConnection,
    user_id: i32,
    offset: u64,
    limit: u64,
) -> Result<RatedResults, AppError> {
    let song_ratings = RatingEntity::find()
        .filter(ratings::Column::UserId.eq(user_id))
        .filter(ratings::Column::ItemType.eq("song"))
        .offset(offset)
        .limit(limit)
        .all(db)
        .await?;

    let album_ratings = RatingEntity::find()
        .filter(ratings::Column::UserId.eq(user_id))
        .filter(ratings::Column::ItemType.eq("album"))
        .offset(offset)
        .limit(limit)
        .all(db)
        .await?;

    let song_total = RatingEntity::find()
        .filter(ratings::Column::UserId.eq(user_id))
        .filter(ratings::Column::ItemType.eq("song"))
        .count(db)
        .await?;

    let album_total = RatingEntity::find()
        .filter(ratings::Column::UserId.eq(user_id))
        .filter(ratings::Column::ItemType.eq("album"))
        .count(db)
        .await?;

    // ---- Resolve names -----------------------------------------------------

    let song_ids: Vec<i32> = song_ratings.iter().map(|r| r.item_id).collect();
    let song_infos: HashMap<i32, (String, String)> = if !song_ids.is_empty() {
        let songs = SongEntity::find()
            .filter(songs::Column::Id.is_in(&song_ids))
            .all(db)
            .await?;
        let a_ids: Vec<i32> = songs.iter().map(|s| s.artist_id).collect();
        let a_names: HashMap<i32, String> = ArtistEntity::find()
            .filter(artists::Column::Id.is_in(a_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|a| (a.id, a.name))
            .collect();
        songs
            .into_iter()
            .map(|s| {
                (
                    s.id,
                    (
                        s.title,
                        a_names.get(&s.artist_id).cloned().unwrap_or_default(),
                    ),
                )
            })
            .collect()
    } else {
        HashMap::new()
    };

    let album_ids: Vec<i32> = album_ratings.iter().map(|r| r.item_id).collect();
    let album_infos: HashMap<i32, (String, String)> = if !album_ids.is_empty() {
        let albums = AlbumEntity::find()
            .filter(albums::Column::Id.is_in(&album_ids))
            .all(db)
            .await?;
        let a_ids: Vec<i32> = albums.iter().map(|a| a.artist_id).collect();
        let a_names: HashMap<i32, String> = ArtistEntity::find()
            .filter(artists::Column::Id.is_in(a_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|a| (a.id, a.name))
            .collect();
        albums
            .into_iter()
            .map(|a| {
                (
                    a.id,
                    (
                        a.name,
                        a_names.get(&a.artist_id).cloned().unwrap_or_default(),
                    ),
                )
            })
            .collect()
    } else {
        HashMap::new()
    };

    // ---- Build response ----------------------------------------------------

    Ok(RatedResults {
        songs: song_ratings
            .into_iter()
            .map(|r| {
                let (title, artist_name) = song_infos
                    .get(&r.item_id)
                    .cloned()
                    .unwrap_or_default();
                RatedItem {
                    id: r.item_id,
                    name: title,
                    artist_name: Some(artist_name),
                    rating: r.rating,
                    rated_at: r.rated_at,
                }
            })
            .collect(),
        albums: album_ratings
            .into_iter()
            .map(|r| {
                let (name, artist_name) = album_infos
                    .get(&r.item_id)
                    .cloned()
                    .unwrap_or_default();
                RatedItem {
                    id: r.item_id,
                    name,
                    artist_name: Some(artist_name),
                    rating: r.rating,
                    rated_at: r.rated_at,
                }
            })
            .collect(),
        song_total,
        album_total,
    })
}

/// Return scrobble history (submissions only) for a user, newest first.
pub async fn get_history(
    db: &DatabaseConnection,
    user_id: i32,
    offset: u64,
    limit: u64,
) -> Result<HistoryResults, AppError> {
    let total = ScrobbleEntity::find()
        .filter(scrobbles::Column::UserId.eq(user_id))
        .filter(scrobbles::Column::Submission.eq(1))
        .count(db)
        .await?;

    let entries = ScrobbleEntity::find()
        .filter(scrobbles::Column::UserId.eq(user_id))
        .filter(scrobbles::Column::Submission.eq(1))
        .order_by(scrobbles::Column::PlayedAt, Order::Desc)
        .offset(offset)
        .limit(limit)
        .all(db)
        .await?;

    // ---- Resolve song / artist / album names --------------------------------

    let song_ids: Vec<i32> = entries.iter().map(|e| e.song_id).collect();
    let song_infos: HashMap<i32, (String, String, Option<String>)> =
        if !song_ids.is_empty() {
            let songs = SongEntity::find()
                .filter(songs::Column::Id.is_in(&song_ids))
                .all(db)
                .await?;
            let a_ids: Vec<i32> = songs.iter().map(|s| s.artist_id).collect();
            let al_ids: Vec<i32> = songs.iter().filter_map(|s| s.album_id).collect();

            let a_names: HashMap<i32, String> = ArtistEntity::find()
                .filter(artists::Column::Id.is_in(a_ids))
                .all(db)
                .await?
                .into_iter()
                .map(|a| (a.id, a.name))
                .collect();

            let al_names: HashMap<i32, String> = if !al_ids.is_empty() {
                AlbumEntity::find()
                    .filter(albums::Column::Id.is_in(al_ids))
                    .all(db)
                    .await?
                    .into_iter()
                    .map(|a| (a.id, a.name))
                    .collect()
            } else {
                HashMap::new()
            };

            songs
                .into_iter()
                .map(|s| {
                    (
                        s.id,
                        (
                            s.title,
                            a_names.get(&s.artist_id).cloned().unwrap_or_default(),
                            s.album_id.and_then(|aid| al_names.get(&aid).cloned()),
                        ),
                    )
                })
                .collect()
        } else {
            HashMap::new()
        };

    let results: Vec<HistoryEntry> = entries
        .into_iter()
        .map(|e| {
            let (title, artist_name, album_name) = song_infos
                .get(&e.song_id)
                .cloned()
                .unwrap_or_default();
            HistoryEntry {
                id: e.id,
                song_id: e.song_id,
                title,
                artist_name,
                album_name,
                played_at: e.played_at,
            }
        })
        .collect();

    Ok(HistoryResults {
        entries: results,
        total,
    })
}
