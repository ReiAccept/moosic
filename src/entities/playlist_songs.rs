use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "playlist_songs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub playlist_id: i32,
    #[sea_orm(primary_key)]
    pub song_id: i32,
    pub position: i32,
    /// 13-digit Unix millisecond timestamp.
    pub added_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::playlists::Entity",
        from = "Column::PlaylistId",
        to = "super::playlists::Column::Id"
    )]
    Playlist,
    #[sea_orm(
        belongs_to = "super::songs::Entity",
        from = "Column::SongId",
        to = "super::songs::Column::Id"
    )]
    Song,
}

impl ActiveModelBehavior for ActiveModel {}
