use sea_orm::{DatabaseConnection, DbErr};

pub async fn connect(url: &str) -> Result<DatabaseConnection, DbErr> {
    sea_orm::Database::connect(url).await
}
