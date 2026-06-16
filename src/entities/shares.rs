use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "shares")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub owner_id: i32,
    pub item_type: String,
    pub item_id: i32,
    pub description: Option<String>,
    #[sea_orm(unique)]
    pub token: String,
    pub visit_count: i32,
    /// 13-digit Unix millisecond timestamp.
    pub last_visited_at: Option<i64>,
    /// 13-digit Unix millisecond timestamp.
    pub expires_at: Option<i64>,
    /// 13-digit Unix millisecond timestamp.
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::OwnerId",
        to = "super::users::Column::Id"
    )]
    Owner,
}

impl ActiveModelBehavior for ActiveModel {}
