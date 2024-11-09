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

const EXAMPLE_DATA: &str = r#"{"location":{"id":1,"x":24.0, "y": 12.0}}"#;
const GET_SQL_STATEMENT: &str = "SELECT id, x, y FROM LED_Location WHERE id = ? LIMIT 1";
const DELETE_SQL_STATEMENT: &str = "DELETE FROM LED_Location WHERE id = ? LIMIT 1";
const UPDATE_SQL_STATEMENT: &str = "UPDATE LED_Location SET x = ?, y= ? WHERE id = ?";
const INSERT_SQL_STATEMENT: &str = "INSERT INTO LED_Location (x, y) Values(?, ?)";

#[derive(Clone, FromRow, Debug, Serialize)]
pub struct LedLocation {
    pub id: i32,
    pub x: f64,
    pub y: f64,
}

impl LedLocation {
    fn extract_from_dict(dict: &Value) -> std::result::Result<Self, Value> {
        let x = match dict.get("x") {
            Some(value) => match value.as_f64() {
                Some(value) => value,
                None => return Err(json!({"error":format!("could not convert x entry to a f64")})),
            },
            None => return Err(json!({"error":format!("could not find x")})),
        };
        let y = match dict.get("y") {
            Some(value) => match value.as_f64() {
                Some(value) => value,
                None => return Err(json!({"error":format!("could not convert y entry to a f64")})),
            },
            None => return Err(json!({"error":format!("could not find y")})),
        };

        return Ok(LedLocation { id: -1, x: x, y: y });
    }
}

pub fn router(index: &mut HashMap<&'static str, &str>, state: Arc<AppState>) -> Router {
    let app = Router::new()
        .route("/", post(post_location))
        .route("/", get(get_all_location))
        .route("/:id", get(get_location_id))
        .route("/:id", put(put_location_id))
        .route("/:id", delete(delete_location_id))
        .with_state(state);

    index.insert("/location", "POST");
    index.insert("/location/:id", "GET,PUT,DELETE");
    return app;
}

pub async fn get_location_id(
    extract::Path(frame_id): extract::Path<i32>,
    extract::State(state): extract::State<Arc<AppState>>,
) -> String {
    let frame_results = sqlx::query_as::<_, LedLocation>(GET_SQL_STATEMENT)
        .bind(frame_id)
        .fetch_one(&state.db)
        .await;

    let data: String = match frame_results {
        Ok(value) => serde_json::to_string(&value).unwrap(),
        Err(value) => return json!({"error": value.to_string()}).to_string(),
    };
    return data;
}
pub async fn get_all_location(
    extract::State(state): extract::State<Arc<AppState>>,
) -> String {
    let frame_results = sqlx::query_as::<_, LedLocation>(
        "SELECT id, x, y FROM LED_Location"
    )
        .fetch_all(&state.db)
        .await;

    let data: String = match frame_results {
        Ok(value) => serde_json::to_string(&value).unwrap(),
        Err(value) => return json!({"error": value.to_string()}).to_string(),
    };
    return data;
}

pub async fn delete_location_id(
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

pub async fn put_location_id(
    extract::Path(frame_id): extract::Path<i32>,
    extract::State(state): extract::State<Arc<AppState>>,
    payload: String,
) -> String {
    let json_payload: Value = match serde_json::from_str(&payload) {
        Ok(result) => result,
        Err(error) => {
            return json!({"error":format!("{error:?}"), "example":EXAMPLE_DATA}).to_string()
        }
    };

    let frame_dict = match json_payload.get("frame") {
        Some(value) => value,
        None => return json!({"error":"frame_data not found", "example":EXAMPLE_DATA}).to_string(),
    };

    let mut led: LedLocation = match LedLocation::extract_from_dict(frame_dict) {
        Ok(value) => value,
        Err(value) => return value.to_string(),
    };
    led.id = frame_id;

    let led_results = sqlx::query(UPDATE_SQL_STATEMENT)
        .bind(led.x)
        .bind(led.y)
        .bind(led.id)
        .execute(&state.db)
        .await;

    match led_results {
        Ok(value) => return json!({"result": format!("{value:?}")}).to_string(),
        Err(value) => return json!({"error":format!("{value:?}")}).to_string(),
    };
}

pub async fn post_location(
    extract::State(state): extract::State<Arc<AppState>>,
    payload: String,
) -> String {
    let json_payload: Value = match serde_json::from_str(&payload) {
        Ok(result) => result,
        Err(error) => {
            return json!({"error":format!("{error:?}"), "example":EXAMPLE_DATA}).to_string()
        }
    };

    let frame_dict = match json_payload.get("location") {
        Some(value) => value,
        None => return json!({"error":"location not found", "example":EXAMPLE_DATA}).to_string(),
    };

    let led: LedLocation = match LedLocation::extract_from_dict(frame_dict) {
        Ok(value) => value,
        Err(value) => return value.to_string(),
    };

    let led_results = sqlx::query(INSERT_SQL_STATEMENT)
        .bind(led.x)
        .bind(led.y)
        .execute(&state.db)
        .await;

    match led_results {
        Ok(value) => return json!({"result": format!("{value:?}")}).to_string(),
        Err(value) => return json!({"error":format!("{value:?}")}).to_string(),
    };
}
