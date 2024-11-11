use axum::{
    extract,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Router,
};
use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use sqlx::FromRow;

use crate::database::initialize::AppState;

use super::animation;

const EXAMPLE_DATA: &str = r#"{"frame":{"parent_id":1,"frame_id":1, "data":"[1,2,3]"}}"#;
const GET_SQL_STATEMENT: &str = "SELECT id, parent_id, frame_id, data FROM Frames WHERE id = ?";
const DELETE_SQL_STATEMENT: &str = "DELETE FROM Frames WHERE id = ?";
const UPDATE_SQL_STATEMENT: &str =
    "UPDATE Frames SET parent_id = ?, frame_id= ?, data= ? WHERE id = ?";
const INSERT_SQL_STATEMENT: &str = "INSERT INTO Frames (parent_id, frame_id, data) Values(?, ?, ?)";

#[derive(Clone, FromRow, Debug, Serialize, Deserialize)]
pub struct Frame {
    pub id: i32,
    pub parent_id: i64,
    pub frame_id: i64,
    pub data: String,
}

#[allow(dead_code)]
impl Frame {
    pub fn new() -> Self {
        Frame {
            id: -1,
            parent_id: -1,
            frame_id: -1,
            data: "[]".to_owned(),
        }
    }
    pub fn new_with_color(color: u32, size: usize) -> Self {
        let repeated = std::iter::repeat(color.to_string())
            .take(size)
            .collect::<Vec<_>>()
            .join(",");
        Frame {
            id: -1,
            parent_id: -1,
            frame_id: -1,
            data: format!("[{}]", repeated),
        }
    }

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

    pub fn data_out(self: &Self) -> Vec<u32> {
        let vec: Vec<u32> = serde_json::from_str(&self.data).unwrap_or(Vec::new());
        return vec;
    }
}

pub fn router(index: &mut HashMap<&'static str, &str>, state: Arc<AppState>) -> Router {
    let app = Router::new()
        .route("/", post(post_frame))
        .route("/", get(get_all_frame))
        .route("/:id", get(get_frame_id))
        .route("/:id", put(put_frame_id))
        .route("/:id", delete(delete_frame_id))
        .route("/show/:id", get(show_frame_id))
        .with_state(state);

    index.insert("/frame", "GET,POST");
    index.insert("/frame/:id", "GET,PUT,DELETE");
    index.insert("/frame/show/:id", "GET");
    return app;
}

pub async fn get_frame_id(
    extract::Path(frame_id): extract::Path<i32>,
    extract::State(state): extract::State<Arc<AppState>>,
) -> Response {
    let frame_results = sqlx::query_as::<_, Frame>(GET_SQL_STATEMENT)
        .bind(frame_id)
        .fetch_one(&state.db)
        .await;
    let data = match frame_results {
        Ok(value) => value,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                json!({"error": format!("{error:?}"), "example":EXAMPLE_DATA}).to_string(),
            )
                .into_response()
        }
    };

    return serde_json::to_string(&data).unwrap().into_response();
}
pub async fn show_frame_id(
    extract::Path(frame_id): extract::Path<i32>,
    extract::State(state): extract::State<Arc<AppState>>,
) -> Response {
    let frame_results = sqlx::query_as::<_, Frame>(GET_SQL_STATEMENT)
        .bind(frame_id)
        .fetch_one(&state.db)
        .await;
    let data = match frame_results {
        Ok(value) => value,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                json!({"error": format!("{error:?}"), "example":EXAMPLE_DATA}).to_string(),
            )
                .into_response()
        }
    };
    let meta_frame = animation::get_frame_data(data.parent_id as i32, &state.db).unwrap();

    let mut ani = animation::Animation::from(meta_frame);
    ani.frames.push(data.clone());

    state
        .send_to_controller
        .send(ani)
        .await
        .expect("Could not send data");

    return serde_json::to_string(&data).unwrap().into_response();
}

pub async fn get_all_frame(extract::State(state): extract::State<Arc<AppState>>) -> Response {
    let frame_results =
        sqlx::query_as::<_, Frame>("SELECT id, parent_id, frame_id, data FROM Frames")
            .fetch_all(&state.db)
            .await;
    let data = match frame_results {
        Ok(value) => value,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": format!("{error:?}"), "example":EXAMPLE_DATA}).to_string(),
            )
                .into_response()
        }
    };

    return serde_json::to_string(&data).unwrap().into_response();
}

pub async fn post_frame(
    extract::State(state): extract::State<Arc<AppState>>,
    payload: String,
) -> Response {
    let json_payload: Value = match serde_json::from_str(&payload) {
        Ok(result) => result,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                json!({"error":format!("{error:?}"), "example":EXAMPLE_DATA}).to_string(),
            )
                .into_response()
        }
    };

    let frame_dict = match json_payload.get("frame") {
        Some(value) => value,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                json!({"error":"frame_data not found", "example":EXAMPLE_DATA}).to_string(),
            )
                .into_response()
        }
    };

    let frame = match Frame::extract_from_dict(&frame_dict) {
        Ok(value) => value,
        Err(error) => return (StatusCode::BAD_REQUEST, error.to_string()).into_response(),
    };

    let frame_results = sqlx::query(INSERT_SQL_STATEMENT)
        .bind(frame.parent_id)
        .bind(frame.frame_id)
        .bind(frame.data)
        .execute(&state.db)
        .await;

    match frame_results {
        Ok(stats) => {
            return json!({"id": stats.last_insert_rowid()})
                .to_string()
                .into_response()
        }
        Err(stats) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": stats.to_string()}).to_string(),
            )
                .into_response()
        }
    };
}

pub async fn put_frame_id(
    extract::Path(frame_id): extract::Path<i32>,
    extract::State(state): extract::State<Arc<AppState>>,
    payload: String,
) -> Response {
    let json_payload: Value = match serde_json::from_str(&payload) {
        Ok(result) => result,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                json!({"error":format!("{error:?}"), "example":EXAMPLE_DATA}).to_string(),
            )
                .into_response()
        }
    };

    let frame_dict = match json_payload.get("frame") {
        Some(value) => value,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                json!({"error":"frame_data not found", "example":EXAMPLE_DATA}).to_string(),
            )
                .into_response()
        }
    };

    let mut frame: Frame = match Frame::extract_from_dict(frame_dict) {
        Ok(value) => value,
        Err(value) => return (StatusCode::BAD_REQUEST, value.to_string()).into_response(),
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
        Ok(value) => {
            return json!({"result": format!("{value:?}")})
                .to_string()
                .into_response()
        }
        Err(value) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error":format!("{value:?}")}).to_string(),
            )
                .into_response()
        }
    };
}

pub async fn delete_frame_id(
    extract::Path(frame_id): extract::Path<i32>,
    extract::State(state): extract::State<Arc<AppState>>,
) -> Response {
    let frame_results = sqlx::query(DELETE_SQL_STATEMENT)
        .bind(frame_id)
        .execute(&state.db)
        .await;

    let data = match frame_results {
        Ok(value) => value,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                json!({"error": format!("{error:?}"), "example":EXAMPLE_DATA}).to_string(),
            )
                .into_response();
        }
    };

    return json!({"value": format!("{data:?}")})
        .to_string()
        .into_response();
}
