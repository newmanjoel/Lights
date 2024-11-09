use std::path::Path;

use serde::Serialize;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use sqlx::FromRow;
use sqlx::{query, Error};

use crate::frame::Frame;




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

    // load_dummy_data(&pool).await;
    // read_dummy_data(&pool).await;
    return Ok(pool);
}

pub async fn create_table_structure(pool: &SqlitePool) -> Result<(), Error> {
    let frame_metadata_sqlite = "
    CREATE TABLE IF NOT EXISTS Frame_Metadata(
        id INTEGER PRIMARY KEY,
        name TEXT,
        speed REAL
    )";

    let frame_sqlite = "CREATE TABLE IF NOT EXISTS Frames(
        id INTEGER PRIMARY KEY,
        parent_id INTEGER,
        data TEXT,
        FOREIGN KEY (parent_id) REFERENCES Frame_Metadata(id)
    )";

    query(frame_metadata_sqlite).execute(pool).await?;
    query(frame_sqlite).execute(pool).await?;
    return Ok(());
}

async fn load_dummy_data(pool: &SqlitePool) {
    let parent_id = 2;
    // let result = sqlx::query("INSERT INTO Frame_Metadata (id, name, speed) VALUES (?, ?, ?)")
    //     .bind(parent_id)
    //     .bind("Some descriptor")
    //     .bind(32.3)
    //     .execute(pool)
    //     .await
    //     .unwrap();
    // println!("Query result: {:?}", result);

    let result = sqlx::query("INSERT INTO Frames (parent_id, data) VALUES (?, ?)")
        .bind(parent_id)
        .bind("[1,2,3]")
        .execute(pool)
        .await
        .unwrap();
    println!("Query result: {:?}", result);
}

pub async fn read_dummy_data(pool: &SqlitePool) -> Result<Vec<Frame>, Error> {
    let frame_results = sqlx::query_as::<_, Frame>("SELECT id, parent_id, data FROM Frames")
        .fetch_all(pool)
        .await
        .unwrap();
    for frame in &frame_results {
        println!("{frame:?}");
    }
    return Ok(frame_results);
}


