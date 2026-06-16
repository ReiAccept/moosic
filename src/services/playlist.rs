use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use sea_orm::{
    ActiveModelTrait, ActiveValue::*, ColumnTrait, Condition, EntityTrait, IntoActiveModel,
    Order, QueryFilter, QueryOrder,
};
use serde::Serialize;

use crate::entities::prelude::*;
use crate::entities::{playlist_songs, playlists, songs, artists, albums, stars, users};
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

/// Summary returned by `list_visible`.
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

/// Single song entry returned by detail / mutating endpoints.
#[derive(Serialize)]
pub struct PlaylistSongItem {
    pub position: i32,
    pub song_id: i32,
    pub title: String,
    pub artist_name: String,
    pub album_name: Option<String>,
    pub duration_secs: i32,
    pub starred: Option<i64>,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Load the full song list for a playlist, resolving names and star status.
async fn get_playlist_song_items(
    db: &DatabaseConnection,
    playlist_id: i32,
    user_id: i32,
) -> Result<Vec<PlaylistSongItem>, AppError> {
    let entries = PlaylistSongEntity::find()
        .filter(playlist_songs::Column::PlaylistId.eq(playlist_id))
        .order_by(playlist_songs::Column::Position, Order::Asc)
        .all(db)
        .await?;

    if entries.is_empty() {
        return Ok(Vec::new());
    }

    let song_ids: Vec<i32> = entries.iter().map(|e| e.song_id).collect();

    let song_models = SongEntity::find()
        .filter(songs::Column::Id.is_in(song_ids.clone()))
        .all(db)
        .await?;

    let artist_ids: Vec<i32> = song_models.iter().map(|s| s.artist_id).collect();
    let artists_map: HashMap<i32, String> = ArtistEntity::find()
        .filter(artists::Column::Id.is_in(artist_ids))
        .all(db)
        .await?
        .into_iter()
        .map(|a| (a.id, a.name))
        .collect();

    let album_ids: Vec<i32> = song_models.iter().filter_map(|s| s.album_id).collect();
    let albums_map: HashMap<i32, String> = if !album_ids.is_empty() {
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

    let starred_map: HashMap<i32, i64> = StarEntity::find()
        .filter(stars::Column::UserId.eq(user_id))
        .filter(stars::Column::ItemType.eq("song"))
        .filter(stars::Column::ItemId.is_in(song_ids))
        .all(db)
        .await?
        .into_iter()
        .map(|s| (s.item_id, s.starred_at))
        .collect();

    let songs_map: HashMap<i32, songs::Model> =
        song_models.into_iter().map(|s| (s.id, s)).collect();

    Ok(entries
        .into_iter()
        .map(|e| {
            let song = songs_map.get(&e.song_id);
            PlaylistSongItem {
                position: e.position,
                song_id: e.song_id,
                title: song.map(|s| s.title.clone()).unwrap_or_default(),
                artist_name: song
                    .and_then(|s| artists_map.get(&s.artist_id))
                    .cloned()
                    .unwrap_or_default(),
                album_name: song
                    .and_then(|s| s.album_id)
                    .and_then(|aid| albums_map.get(&aid))
                    .cloned(),
                duration_secs: song.map(|s| s.duration_secs).unwrap_or(0),
                starred: starred_map.get(&e.song_id).copied(),
            }
        })
        .collect())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// List playlists visible to the given user (owned or public).
pub async fn list_visible(
    db: &DatabaseConnection,
    user_id: i32,
) -> Result<Vec<PlaylistSummary>, AppError> {
    let playlists = PlaylistEntity::find()
        .filter(
            Condition::any()
                .add(playlists::Column::OwnerId.eq(user_id))
                .add(playlists::Column::IsPublic.eq(1)),
        )
        .all(db)
        .await?;

    if playlists.is_empty() {
        return Ok(Vec::new());
    }

    let playlist_ids: Vec<i32> = playlists.iter().map(|p| p.id).collect();

    // Gather song<->playlist associations for counting / duration.
    let all_entries = PlaylistSongEntity::find()
        .filter(playlist_songs::Column::PlaylistId.is_in(playlist_ids))
        .all(db)
        .await?;

    let song_ids: Vec<i32> = all_entries.iter().map(|e| e.song_id).collect();
    let song_durations: HashMap<i32, i32> = if !song_ids.is_empty() {
        SongEntity::find()
            .filter(songs::Column::Id.is_in(song_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|s| (s.id, s.duration_secs))
            .collect()
    } else {
        HashMap::new()
    };

    let mut playlist_stats: HashMap<i32, (i64, i64)> = HashMap::new();
    for entry in &all_entries {
        let (count, duration) = playlist_stats.entry(entry.playlist_id).or_insert((0, 0));
        *count += 1;
        if let Some(d) = song_durations.get(&entry.song_id) {
            *duration += *d as i64;
        }
    }

    // Resolve owner names.
    let owner_ids: Vec<i32> = playlists.iter().map(|p| p.owner_id).collect();
    let owner_names: HashMap<i32, String> = if !owner_ids.is_empty() {
        UserEntity::find()
            .filter(users::Column::Id.is_in(owner_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|u| (u.id, u.username))
            .collect()
    } else {
        HashMap::new()
    };

    Ok(playlists
        .into_iter()
        .map(|p| {
            let (song_count, duration_secs) =
                playlist_stats.get(&p.id).copied().unwrap_or((0, 0));
            PlaylistSummary {
                id: p.id,
                name: p.name,
                owner_name: owner_names
                    .get(&p.owner_id)
                    .cloned()
                    .unwrap_or_default(),
                is_public: p.is_public != 0,
                song_count,
                duration_secs,
                created_at: p.created_at,
                updated_at: p.updated_at,
            }
        })
        .collect())
}

/// Get a single playlist with its full song listing.
///
/// Returns the playlist model and the resolved song items.
pub async fn get_detail(
    db: &DatabaseConnection,
    id: i32,
    user_id: i32,
) -> Result<(playlists::Model, Vec<PlaylistSongItem>), AppError> {
    let playlist = PlaylistEntity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Playlist not found"))?;

    if playlist.owner_id != user_id && playlist.is_public == 0 {
        return Err(AppError::not_found("Playlist not found"));
    }

    let items = get_playlist_song_items(db, id, user_id).await?;
    Ok((playlist, items))
}

/// Create a new playlist.
pub async fn create(
    db: &DatabaseConnection,
    owner_id: i32,
    name: &str,
    comment: Option<&str>,
    is_public: bool,
    song_ids: &[i32],
) -> Result<playlists::Model, AppError> {
    let now = now_ms();
    let model = playlists::ActiveModel {
        name: Set(name.to_owned()),
        owner_id: Set(owner_id),
        comment: Set(comment.map(|s| s.to_owned())),
        is_public: Set(if is_public { 1 } else { 0 }),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    let playlist = model.insert(db).await?;

    if !song_ids.is_empty() {
        let now = now_ms();
        let entries: Vec<playlist_songs::ActiveModel> = song_ids
            .iter()
            .enumerate()
            .map(|(i, song_id)| playlist_songs::ActiveModel {
                playlist_id: Set(playlist.id),
                song_id: Set(*song_id),
                position: Set(i as i32 + 1),
                added_at: Set(now),
            })
            .collect();
        PlaylistSongEntity::insert_many(entries).exec(db).await?;
    }

    Ok(playlist)
}

/// Update playlist metadata (name, comment, is_public).
pub async fn update(
    db: &DatabaseConnection,
    id: i32,
    owner_id: i32,
    name: Option<String>,
    comment: Option<Option<String>>,
    is_public: Option<bool>,
) -> Result<playlists::Model, AppError> {
    let playlist = PlaylistEntity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Playlist not found"))?;

    if playlist.owner_id != owner_id {
        return Err(AppError::forbidden("You do not own this playlist"));
    }

    let mut ac = playlist.into_active_model();

    if let Some(name) = name {
        ac.name = Set(name);
    }
    if let Some(comment) = comment {
        ac.comment = Set(comment);
    }
    if let Some(is_public) = is_public {
        ac.is_public = Set(if is_public { 1 } else { 0 });
    }
    ac.updated_at = Set(now_ms());

    Ok(ac.update(db).await?)
}

/// Delete a playlist and its song associations (CASCADE).
pub async fn delete(
    db: &DatabaseConnection,
    id: i32,
    owner_id: i32,
) -> Result<(), AppError> {
    let playlist = PlaylistEntity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Playlist not found"))?;

    if playlist.owner_id != owner_id {
        return Err(AppError::forbidden("You do not own this playlist"));
    }

    PlaylistEntity::delete_by_id(id).exec(db).await?;
    Ok(())
}

/// Append songs to the end of a playlist.
pub async fn add_songs(
    db: &DatabaseConnection,
    id: i32,
    owner_id: i32,
    song_ids: &[i32],
) -> Result<Vec<PlaylistSongItem>, AppError> {
    let playlist = PlaylistEntity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Playlist not found"))?;

    if playlist.owner_id != owner_id {
        return Err(AppError::forbidden("You do not own this playlist"));
    }

    // Determine the current max position so we append after it.
    let max_position = PlaylistSongEntity::find()
        .filter(playlist_songs::Column::PlaylistId.eq(id))
        .order_by(playlist_songs::Column::Position, Order::Desc)
        .one(db)
        .await?
        .map(|e| e.position)
        .unwrap_or(0);

    let now = now_ms();
    let entries: Vec<playlist_songs::ActiveModel> = song_ids
        .iter()
        .enumerate()
        .map(|(i, song_id)| playlist_songs::ActiveModel {
            playlist_id: Set(id),
            song_id: Set(*song_id),
            position: Set(max_position + 1 + i as i32),
            added_at: Set(now),
        })
        .collect();

    PlaylistSongEntity::insert_many(entries).exec(db).await?;

    get_playlist_song_items(db, id, owner_id).await
}

/// Remove songs from a playlist and renumber remaining positions.
pub async fn remove_songs(
    db: &DatabaseConnection,
    id: i32,
    owner_id: i32,
    song_ids: &[i32],
) -> Result<Vec<PlaylistSongItem>, AppError> {
    let playlist = PlaylistEntity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Playlist not found"))?;

    if playlist.owner_id != owner_id {
        return Err(AppError::forbidden("You do not own this playlist"));
    }

    if !song_ids.is_empty() {
        PlaylistSongEntity::delete_many()
            .filter(playlist_songs::Column::PlaylistId.eq(id))
            .filter(playlist_songs::Column::SongId.is_in(song_ids.iter().copied()))
            .exec(db)
            .await?;
    }

    // Renumber remaining positions sequentially from 1.
    let remaining = PlaylistSongEntity::find()
        .filter(playlist_songs::Column::PlaylistId.eq(id))
        .order_by(playlist_songs::Column::Position, Order::Asc)
        .all(db)
        .await?;

    for (i, entry) in remaining.iter().enumerate() {
        let mut ac = entry.clone().into_active_model();
        ac.position = Set(i as i32 + 1);
        ac.update(db).await?;
    }

    get_playlist_song_items(db, id, owner_id).await
}

/// Replace the entire song list of a playlist with a new ordering.
pub async fn reorder(
    db: &DatabaseConnection,
    id: i32,
    owner_id: i32,
    song_ids: &[i32],
) -> Result<Vec<PlaylistSongItem>, AppError> {
    let playlist = PlaylistEntity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Playlist not found"))?;

    if playlist.owner_id != owner_id {
        return Err(AppError::forbidden("You do not own this playlist"));
    }

    // Delete all existing entries.
    PlaylistSongEntity::delete_many()
        .filter(playlist_songs::Column::PlaylistId.eq(id))
        .exec(db)
        .await?;

    // Re-insert with positions matching the supplied order.
    let now = now_ms();
    let entries: Vec<playlist_songs::ActiveModel> = song_ids
        .iter()
        .enumerate()
        .map(|(i, song_id)| playlist_songs::ActiveModel {
            playlist_id: Set(id),
            song_id: Set(*song_id),
            position: Set(i as i32 + 1),
            added_at: Set(now),
        })
        .collect();

    PlaylistSongEntity::insert_many(entries).exec(db).await?;

    get_playlist_song_items(db, id, owner_id).await
}
