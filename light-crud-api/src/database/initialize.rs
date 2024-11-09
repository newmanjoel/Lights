use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use sqlx::{query, Error};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: SqlitePool,
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
