use sea_orm::{DatabaseConnection, DbErr};

/// Connect to a SQLite database.
pub async fn connect(url: &str) -> Result<DatabaseConnection, DbErr> {
    sea_orm::Database::connect(url).await
}
