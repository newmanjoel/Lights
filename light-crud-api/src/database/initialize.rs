use axum::routing::get;
use axum::Router;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use sqlx::{query, Error};
use std::collections::HashMap;
use std::path::Path;

use crate::config::Config;

use super::{frame, frame_data, location};

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: SqlitePool,
    pub send_to_controller: tokio::sync::mpsc::Sender<frame::Frame>,
}

pub async fn setup(config: &Config) -> Router {
    let mut index: HashMap<&'static str, &str> = HashMap::new();

    let filepath = Path::new(config.database.file_path.as_str());
    let pool = get_or_create_sqlite_database(filepath).await.unwrap();
    let state: std::sync::Arc<AppState> = std::sync::Arc::new(AppState {
        db: pool,
        send_to_controller: config.sending_channel.clone(),
    });
    let frame_routes = frame::router(&mut index, state.clone());
    let frame_data_routes = frame_data::router(&mut index, state.clone());
    let location_routes = location::router(&mut index, state.clone());

    let app: Router = Router::new()
        .route(
            "/",
            get(|| async move { return serde_json::to_string_pretty(&index).unwrap().to_string() }),
        )
        .nest("/frame", frame_routes)
        .nest("/frame_data", frame_data_routes)
        .nest("/location", location_routes);
    return app;
}

pub async fn get_or_create_sqlite_database(filepath: &Path) -> Result<SqlitePool, Error> {
    let options = SqliteConnectOptions::new()
        .filename(filepath)
        .create_if_missing(true);

    let pool = match SqlitePool::connect_with(options).await {
        Ok(pool) => pool,
        Err(error) => panic!("Problem: {error:?}"),
    };
    match create_table_structure(&pool).await {
        Ok(ok) => ok,
        Err(error) => panic!("Problem creating tables: {error:?}"),
    };

    return Ok(pool);
}

pub async fn create_table_structure(pool: &SqlitePool) -> Result<(), Error> {
    let location_sqlite = "
    CREATE TABLE IF NOT EXISTS LED_Location(
        id INTEGER PRIMARY KEY,
        x REAL,
        y REAL,
        UNIQUE(x,y)
    )";

    let frame_metadata_sqlite = "
    CREATE TABLE IF NOT EXISTS Frame_Metadata(
        id INTEGER PRIMARY KEY,
        name TEXT,
        speed REAL,
        UNIQUE(name)
    )";

    let frame_sqlite = "
    CREATE TABLE IF NOT EXISTS Frames(
        id INTEGER PRIMARY KEY,
        parent_id INTEGER,
        frame_id INTEGER,
        data TEXT,
        FOREIGN KEY (parent_id) REFERENCES Frame_Metadata(id),
        UNIQUE(parent_id, frame_id)
    )";

    query(frame_metadata_sqlite).execute(pool).await?;
    query(frame_sqlite).execute(pool).await?;
    query(location_sqlite).execute(pool).await?;
    return Ok(());
}
