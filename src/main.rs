mod config;
mod entities;

use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use config::Config;
use entities::prelude::*;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveValue, Database, DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // load configuration from JSON file
    let config = Config::load();
    tracing::info!("Loaded config: database.url={}", config.database.url);

    // connect to SQLite database
    let db: DatabaseConnection = Database::connect(&config.database.url)
        .await
        .expect("Failed to connect to database");

    // run pending migrations
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Database connected and migrations applied");

    // build our application with a shared database connection
    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user))
        .with_state(db);

    // bind to the configured address and port
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {addr}: {e}"));

    tracing::info!("Listening on {addr}");

    let _ = axum::serve(listener, app).await;
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn create_user(
    axum::extract::State(db): axum::extract::State<DatabaseConnection>,
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), (StatusCode, String)> {
    let username = payload.username;

    let user = UserActiveModel {
        username: ActiveValue::Set(username.clone()),
        ..Default::default()
    };

    let res = UserEntity::insert(user).exec(&db).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to insert user: {e}"),
        )
    })?;

    Ok((
        StatusCode::CREATED,
        Json(User {
            id: res.last_insert_id,
            username,
        }),
    ))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: i32,
    username: String,
}
