use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "lyrics")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub song_id: i32,
    #[sea_orm(column_name = "type")]
    pub type_: String,
    pub content: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::songs::Entity",
        from = "Column::SongId",
        to = "super::songs::Column::Id"
    )]
    Songs,
}

impl ActiveModelBehavior for ActiveModel {}
