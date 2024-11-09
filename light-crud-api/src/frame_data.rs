use axum::extract;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::{collections::HashMap, sync::Arc};

use serde::Serialize;
use serde_json::{json, Value};
use sqlx::FromRow;

use crate::database_stuff::AppState;
// use crate::frame::Frame;

const EXAMPLE_DATA: &str = r#"{"frame_data":{"name":"Some String Name","speed":24.0}}"#;
const GET_SQL_STATEMENT: &str = "SELECT id, name, speed FROM Frame_Metadata WHERE id = ? LIMIT 1";
const DELETE_SQL_STATEMENT: &str = "DELETE FROM Frame_Metadata WHERE id = ? LIMIT 1";
const UPDATE_SQL_STATEMENT: &str = "UPDATE Frame_Metadata SET name = ?, speed= ? WHERE id = ?";
const INSERT_SQL_STATEMENT: &str = "INSERT INTO Frame_Metadata (name, speed) Values(?, ?)";

#[derive(Clone, FromRow, Debug, Serialize)]
pub struct FrameMetadata {
    pub id: i32,
    pub name: String,
    pub speed: f64,
}

impl FrameMetadata {
    fn extract_from_dict(dict: &Value) -> std::result::Result<Self, Value> {
        let name_result = extract_str_from_result(dict, "name");
        let speed_result = extract_f64_from_result(dict, "speed");

        let name = match name_result {
            Ok(value) => value,
            Err(value) => return Err(value),
        };

        let speed = match speed_result {
            Ok(value) => value,
            Err(value) => return Err(value),
        };
        return Ok(FrameMetadata {
            id: -1,
            name: name,
            speed: speed,
        });
    }
}

pub fn router(index: &mut HashMap<&'static str, &str>, state: Arc<AppState>) -> Router {
    let app = Router::new()
        .route("/", post(post_frame_data))
        .route("/", get(get_all_frame_data))
        .route("/:id", get(get_frame_data_id))
        .route("/:id", put(put_frame_data_id))
        .route("/:id", delete(delete_frame_data_id))
        .with_state(state);

    index.insert("/frame_data", "GET,POST");
    index.insert("/frame_data/:id", "GET,PUT,DELETE");
    return app;
}

pub async fn get_all_frame_data(
    extract::State(state): extract::State<Arc<AppState>>,
) -> String {
    let frame_results = sqlx::query_as::<_, FrameMetadata>(
        "SELECT id, name, speed FROM Frame_Metadata"
    )
        .fetch_all(&state.db)
        .await;

    let data: String = match frame_results {
        Ok(value) => serde_json::to_string(&value).unwrap(),
        Err(value) => return json!({"error": value.to_string()}).to_string(),
    };
    return data;
}

pub async fn get_frame_data_id(
    extract::Path(frame_id): extract::Path<i32>,
    extract::State(state): extract::State<Arc<AppState>>,
) -> String {
    let frame_results = sqlx::query_as::<_, FrameMetadata>(GET_SQL_STATEMENT)
        .bind(frame_id)
        .fetch_one(&state.db)
        .await;

    let data: String = match frame_results {
        Ok(value) => serde_json::to_string(&value).unwrap(),
        Err(value) => return json!({"error": value.to_string()}).to_string(),
    };
    return data;
}

pub async fn delete_frame_data_id(
    extract::Path(frame_id): extract::Path<i32>,
    extract::State(state): extract::State<Arc<AppState>>,
) -> String {
    let frame_results = sqlx::query(DELETE_SQL_STATEMENT)
        .bind(frame_id)
        .execute(&state.db)
        .await
        .unwrap();

    return json!({"last insert rowid":frame_results.last_insert_rowid()}).to_string();
}

pub async fn put_frame_data_id(
    extract::Path(frame_id): extract::Path<i32>,
    extract::State(state): extract::State<Arc<AppState>>,
    payload: String,
) -> String {
    let json_payload: Value = match serde_json::from_str(&payload) {
        Ok(result) => result,
        Err(error) => return json!({"error":"parsing json", "payload":payload, "example":EXAMPLE_DATA, "debug":error.to_string()}).to_string(),
    };

    let frame_dict = match extract_frame_from_dict(&json_payload) {
        Ok(arr) => arr,
        Err(value) => return value.to_string(),
    };

    let mut frame: FrameMetadata = match FrameMetadata::extract_from_dict(frame_dict) {
        Ok(value) => value,
        Err(value) => return value.to_string(),
    };
    frame.id = frame_id;

    let frame_results = sqlx::query(UPDATE_SQL_STATEMENT)
        .bind(frame.name)
        .bind(frame.speed)
        .bind(frame.id)
        .execute(&state.db)
        .await;

    match frame_results {
        Ok(value) => return json!({"result": format!("{value:?}")}).to_string(),
        Err(value) => return json!({"error":format!("{value:?}")}).to_string(),
    };
}

fn extract_frame_from_dict(input_dict: &Value) -> std::result::Result<&Value, Value> {
    match input_dict.get("frame_data") {
        Some(arr) => return Ok(arr),
        None => return Err(json!({"error":"frame_data not found", "example":EXAMPLE_DATA})),
    };
}

fn extract_str_from_result(
    input_dict: &Value,
    dict_name: &str,
) -> std::result::Result<String, Value> {
    match input_dict.get(dict_name) {
        Some(value) => match value.as_str() {
            Some(value) => return Ok(value.to_owned()),
            None => {
                return Err(
                    json!({"error":format!("could not convert {dict_name:?} entry to a string")}),
                )
            }
        },
        None => return Err(json!({"error":format!("could not find {dict_name:?} entry")})),
    };
}

fn extract_f64_from_result(input_dict: &Value, dict_name: &str) -> std::result::Result<f64, Value> {
    match input_dict.get(dict_name) {
        Some(value) => match value.as_f64() {
            Some(value) => return Ok(value),
            None => {
                return Err(
                    json!({"error":format!("could not convert {dict_name:?} entry to a f64")}),
                )
            }
        },
        None => return Err(json!({"error":format!("could not find {dict_name:?} entry")})),
    };
}

pub async fn post_frame_data(
    extract::State(state): extract::State<Arc<AppState>>,
    payload: String,
) -> String {
    let json_payload: Value = match serde_json::from_str(&payload) {
        Ok(result) => result,
        Err(error) => return json!({"error":"parsing json", "payload":payload, "example":EXAMPLE_DATA, "debug":error.to_string()}).to_string(),
    };

    let frame_dict = match extract_frame_from_dict(&json_payload) {
        Ok(arr) => arr,
        Err(value) => return value.to_string(),
    };

    let frame: FrameMetadata = match FrameMetadata::extract_from_dict(frame_dict) {
        Ok(value) => value,
        Err(value) => return value.to_string(),
    };

    let frame_results = sqlx::query(INSERT_SQL_STATEMENT)
        .bind(frame.name)
        .bind(frame.speed)
        .execute(&state.db)
        .await;

    match frame_results {
        Ok(stats) => return json!({"id": stats.last_insert_rowid()}).to_string(),
        Err(stats) => return json!({"error": stats.to_string()}).to_string(),
    };
}


