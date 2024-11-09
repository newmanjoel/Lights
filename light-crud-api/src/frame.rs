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

const EXAMPLE_DATA: &str = r#"{"frame":{"parent_id":1,"frame_id":1, "data":"[1,2,3]"}}"#;
const GET_SQL_STATEMENT: &str =
    "SELECT id, parent_id, frame_id, data FROM Frames WHERE id = ? LIMIT 1";
const DELETE_SQL_STATEMENT: &str = "DELETE FROM Frames WHERE id = ? LIMIT 1";
const UPDATE_SQL_STATEMENT: &str =
    "UPDATE Frames SET parent_id = ?, frame_id= ?, data= ? WHERE id = ?";
const INSERT_SQL_STATEMENT: &str = "INSERT INTO Frames (parent_id, frame_id, data) Values(?, ?, ?)";

#[derive(Clone, FromRow, Debug, Serialize)]
pub struct Frame {
    pub id: i32,
    pub parent_id: i64,
    pub frame_id: i64,
    pub data: String,
}

impl Frame {
    fn extract_from_dict(dict: &Value) -> std::result::Result<Self, Value> {
        let parent_id = match dict.get("parent_id") {
            Some(value) => match value.as_i64() {
                Some(value) => value,
                None => {
                    return Err(
                        json!({"error":format!("could not convert parent_id entry to a i64")}),
                    )
                }
            },
            None => return Err(json!({"error":format!("could not find parent_id")})),
        };
        let frame_id = match dict.get("frame_id") {
            Some(value) => match value.as_i64() {
                Some(value) => value,
                None => {
                    return Err(
                        json!({"error":format!("could not convert frame_id entry to a i64")}),
                    )
                }
            },
            None => return Err(json!({"error":format!("could not find frame_id")})),
        };
        let data_str = match dict.get("data") {
            Some(value) => match value.as_str() {
                Some(value) => value,
                None => {
                    return Err(json!({"error":format!("could not convert data entry to a str")}))
                }
            },
            None => return Err(json!({"error":format!("could not find data")})),
        };

        return Ok(Frame {
            id: -1,
            parent_id: parent_id,
            frame_id: frame_id,
            data: data_str.to_owned(),
        });
    }
}

pub fn router(index: &mut HashMap<&'static str, &str>, state: Arc<AppState>) -> Router {
    let app = Router::new()
        .route("/", post(post_frame))
        .route("/:id", get(get_frame_id))
        .route("/:id", put(put_frame_id))
        .route("/:id", delete(delete_frame_id))
        .with_state(state);

    index.insert("/frame", "POST");
    index.insert("/frame/:id", "GET,PUT,DELETE");
    return app;
}

pub async fn get_frame_id(
    extract::Path(frame_id): extract::Path<i32>,
    extract::State(state): extract::State<Arc<AppState>>,
) -> String {
    let frame_results = sqlx::query_as::<_, Frame>(GET_SQL_STATEMENT)
        .bind(frame_id)
        .fetch_one(&state.db)
        .await;
    let data = match frame_results {
        Ok(value) => value,
        Err(error) => {
            return json!({"error": format!("{error:?}"), "example":EXAMPLE_DATA}).to_string()
        }
    };

    return serde_json::to_string(&data).unwrap();
}

pub async fn post_frame(
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

    let frame = match Frame::extract_from_dict(&frame_dict) {
        Ok(value) => value,
        Err(error) => return error.to_string(),
    };

    let frame_results = sqlx::query(INSERT_SQL_STATEMENT)
        .bind(frame.parent_id)
        .bind(frame.frame_id)
        .bind(frame.data)
        .execute(&state.db)
        .await;

    match frame_results {
        Ok(stats) => return json!({"id": stats.last_insert_rowid()}).to_string(),
        Err(stats) => return json!({"error": stats.to_string()}).to_string(),
    };
}

pub async fn put_frame_id(
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

    let mut frame: Frame = match Frame::extract_from_dict(frame_dict) {
        Ok(value) => value,
        Err(value) => return value.to_string(),
    };
    frame.id = frame_id;

    let frame_results = sqlx::query(UPDATE_SQL_STATEMENT)
        .bind(frame.parent_id)
        .bind(frame.frame_id)
        .bind(frame.data)
        .bind(frame.id)
        .execute(&state.db)
        .await;

    match frame_results {
        Ok(value) => return json!({"result": format!("{value:?}")}).to_string(),
        Err(value) => return json!({"error":format!("{value:?}")}).to_string(),
    };
}

pub async fn delete_frame_id(
    extract::Path(frame_id): extract::Path<i32>,
    extract::State(state): extract::State<Arc<AppState>>,
) -> String {
    let frame_results = sqlx::query(DELETE_SQL_STATEMENT)
        .bind(frame_id)
        .execute(&state.db)
        .await;

    let data = match frame_results {
        Ok(value) => value,
        Err(error) => {
            return json!({"error": format!("{error:?}"), "example":EXAMPLE_DATA}).to_string()
        }
    };

    return json!({"value": format!("{data:?}")}).to_string();
}
