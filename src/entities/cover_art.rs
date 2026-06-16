use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "cover_art")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub item_type: String,
    pub item_id: i32,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub file_path: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
