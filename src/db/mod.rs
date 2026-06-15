mod sqlite;

use migration::MigratorTrait;
use sea_orm::DatabaseConnection;

use crate::config::Database;

/// Connect to the database and run pending migrations.
///
/// Returns a backend-agnostic [`DatabaseConnection`] ready to be used
/// as axum state.
pub async fn connect(config: &Database) -> DatabaseConnection {
    let db = match config {
        Database::Sqlite { url } => sqlite::connect(url)
            .await
            .expect("Failed to connect to SQLite database"),
    };

    tracing::info!(
        "Running pending database migrations... type={}",
        config.type_name()
    );
    migration::Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");
    tracing::info!("Migrations applied successfully");

    db
}

impl Database {
    /// Human-readable backend name for logging.
    fn type_name(&self) -> &'static str {
        match self {
            Database::Sqlite { .. } => "sqlite",
        }
    }
}
