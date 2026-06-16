use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "songs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub artist_id: i32,
    pub album_id: Option<i32>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub duration_secs: i32,
    pub bit_rate: Option<i32>,
    pub size_bytes: Option<i64>,
    pub file_format: Option<String>,
    pub content_type: Option<String>,
    pub year: Option<i32>,
    #[sea_orm(unique)]
    pub file_path: String,
    pub has_cover_art: i32,
    pub library_id: i32,
    /// 13-digit Unix millisecond timestamp.
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::artists::Entity",
        from = "Column::ArtistId",
        to = "super::artists::Column::Id"
    )]
    Artists,
    #[sea_orm(
        belongs_to = "super::albums::Entity",
        from = "Column::AlbumId",
        to = "super::albums::Column::Id"
    )]
    Albums,
    #[sea_orm(
        belongs_to = "super::libraries::Entity",
        from = "Column::LibraryId",
        to = "super::libraries::Column::Id"
    )]
    Libraries,
}

impl ActiveModelBehavior for ActiveModel {}
