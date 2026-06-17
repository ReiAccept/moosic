mod sqlite;

use migration::MigratorTrait;
use sea_orm::DatabaseConnection;

use crate::config::Database;

pub async fn connect(config: &Database) -> DatabaseConnection {
    let db = match config {
        Database::Sqlite { url } => sqlite::connect(url)
            .await
            .expect("Failed to connect to SQLite database"),
    };

    tracing::info!(
        "Running pending database migrations... type={}",
        config.kind()
    );
    migration::Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");
    tracing::info!("Migrations applied successfully");

    db
}

impl Database {
    pub fn kind(&self) -> &'static str {
        match self {
            Database::Sqlite { .. } => "sqlite",
        }
    }
}
