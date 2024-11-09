use axum::{extract, routing::get, Router};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;

use serde::Serialize;
use serde_json::{Result, Value};

use sqlx::sqlite::SqlitePool;

use std::path::Path;

mod database_stuff;
use database_stuff::{get_or_create_sqlite_database, read_dummy_data};

#[derive(Clone, Debug)]
struct AppState {
    db: SqlitePool,
}

async fn hello_world() -> &'static str {
    return "Hello World!";
}

async fn do_something(extract::State(state): extract::State<Arc<AppState>>) -> String {
    let frames = match read_dummy_data(&state.db).await {
        Ok(frames) => frames,
        Err(error) => {
            panic!("do_something: {error:?}");
            // return Err(ApiError::InternalServerError);
        }
    };
    println!("do_something: {frames:?}");
    let data: Vec<String> = frames
        .iter()
        .map(|frame| serde_json::to_string(&frame).unwrap())
        .to_owned()
        .collect();
    let owned_data = data.join("|");
    return owned_data;
}

async fn get_frame_id(
    extract::Path(frame_id): extract::Path<i32>,
    extract::State(state): extract::State<Arc<AppState>>,
) -> String {
    let frames = match read_dummy_data(&state.db).await {
        Ok(frames) => frames,
        Err(error) => {
            panic!("do_something: {error:?}");
            // return Err(ApiError::InternalServerError);
        }
    };
    println!("do_something: {frames:?}");
    let data: Vec<String> = frames
        .iter()
        .map(|frame| serde_json::to_string(&frame).unwrap())
        .to_owned()
        .collect();
    let owned_data = data.join("|") + &frame_id.to_string();
    return owned_data;
}

async fn set_frame_id(
    extract::Path(frame_id): extract::Path<i32>,
    extract::State(state): extract::State<Arc<AppState>>,
    payload: String,
) -> String {
    let json_payload: Value = match serde_json::from_str(&payload){
        Ok(result) => result,
        Err(error) => return format!("Error parsing Json: {error:?}"),
    };
    return format!("Setting data for id: {frame_id:?} with payload of: {json_payload:?}");
}

#[tokio::main]
async fn main() {
    let filepath = Path::new("/home/joel/GH/Lights/db/sqlite.db");
    let pool = get_or_create_sqlite_database(filepath).await.unwrap();
    let state = Arc::new(AppState { db: pool });
    println!("{state:?}");

    let app = Router::new()
        .route("/", get(hello_world))
        .route("/do_something", get(do_something))
        .route("/do_id/:id", get(get_frame_id).post(set_frame_id))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curl() {
        // curl -X POST http://localhost:3000/do_id/3 -H "Content-Type: application/json" -d '{"key1":[1,2,3.3]}'
        // Setting data for id: 3 with payload of: Object {"key1": Array [Number(1), Number(2), Number(3.3)]}%   
        assert!(true);
    }
}