use axum::{
    extract,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Router,
};
use futures::executor::block_on;
use std::{collections::HashMap, sync::Arc};

use serde::Serialize;
use serde_json::{json, Value};
use sqlx::{FromRow, Pool, Sqlite};

use crate::database::initialize::AppState;

// use crate::frame::Frame;

const EXAMPLE_DATA: &str = r#"{"frame_data":{"name":"Some String Name","speed":24.0}}"#;
// const GET_SQL_STATEMENT: &str = "SELECT id, name, speed FROM Frame_Metadata WHERE id = ? LIMIT 1";
// const DELETE_SQL_STATEMENT: &str = "DELETE FROM Frame_Metadata WHERE id = ? LIMIT 1";
// const UPDATE_SQL_STATEMENT: &str = "UPDATE Frame_Metadata SET name = ?, speed= ? WHERE id = ?";
// const INSERT_SQL_STATEMENT: &str = "INSERT INTO Frame_Metadata (name, speed) Values(?, ?)";

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

    pub fn get_from_db(id: i32, db: &Pool<Sqlite>) -> Result<Self, sqlx::Error> {
        let result = block_on(
            sqlx::query_as::<_, Self>("SELECT id, name, speed FROM Frame_Metadata WHERE id = ?")
                .bind(id)
                .fetch_one(db),
        );
        return result;
    }

    pub fn update_in_db(self: &Self, db: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        let result = block_on(
            sqlx::query("UPDATE Frame_Metadata SET name = ?, speed= ? WHERE id = ?")
                .bind(self.name.clone())
                .bind(self.speed)
                .bind(self.id)
                .execute(db),
        );
        return match result {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        };
    }

    pub fn insert_in_db(self: &Self, db: &Pool<Sqlite>) -> Result<Self, sqlx::Error> {
        let result = block_on(
            sqlx::query("INSERT INTO Frame_Metadata (name, speed) Values(?, ?)")
                .bind(self.name.clone())
                .bind(self.speed)
                .execute(db),
        );

        return match result {
            Ok(value) => Ok({
                let mut new_frame_metadata = self.clone();
                new_frame_metadata.id = value.last_insert_rowid() as i32;
                new_frame_metadata
            }),
            Err(err) => Err(err),
        };
    }

    pub fn delete_in_db(id: i32, db: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        let result = block_on(
            sqlx::query("DELETE FROM Frame_Metadata WHERE id = ?")
                .bind(id)
                .execute(db),
        );

        return match result {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        };
    }

    #[allow(dead_code)]
    pub fn get_all_from_db(db: &Pool<Sqlite>) -> Vec<Self> {
        let frame_meta_results = block_on(
            sqlx::query_as::<_, FrameMetadata>("SELECT id, name, speed FROM Frame_Metadata")
                .fetch_all(db),
        );

        match frame_meta_results {
            Ok(result) => return result,
            Err(_) => return Vec::new(),
        }
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

fn extract_json_frame(payload: String) -> Result<FrameMetadata, Response> {
    let json_payload: Value = match serde_json::from_str(&payload) {
        Ok(result) => result,
        Err(error) => return Err((StatusCode::BAD_REQUEST, json!({"error":"parsing json", "payload":payload, "example":EXAMPLE_DATA, "debug":error.to_string()}).to_string()).into_response()),
    };

    let frame_dict = match json_payload.get("frame_data") {
        Some(arr) => arr,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                json!({"error":"frame_data not found", "example":EXAMPLE_DATA}).to_string(),
            )
                .into_response())
        }
    };
    let frame: FrameMetadata = match FrameMetadata::extract_from_dict(frame_dict) {
        Ok(value) => value,
        Err(value) => return Err((StatusCode::BAD_REQUEST, value.to_string()).into_response()),
    };
    return Ok(frame);
}

pub async fn get_all_frame_data(extract::State(state): extract::State<Arc<AppState>>) -> Response {
    // TODO: add this to the frame data impl
    let frame_results =
        sqlx::query_as::<_, FrameMetadata>("SELECT id, name, speed FROM Frame_Metadata")
            .fetch_all(&state.db)
            .await;

    match frame_results {
        Ok(value) => return serde_json::to_string(&value).unwrap().into_response(),
        Err(value) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": value.to_string()}).to_string(),
            )
                .into_response()
        }
    };
}

pub async fn get_frame_data_id(
    extract::Path(database_id): extract::Path<i32>,
    extract::State(state): extract::State<Arc<AppState>>,
) -> Response {
    let frame_results = FrameMetadata::get_from_db(database_id, &state.db);

    match frame_results {
        Ok(value) => return serde_json::to_string(&value).unwrap().into_response(),
        Err(value) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": value.to_string()}).to_string(),
            )
                .into_response()
        }
    };
}

pub async fn delete_frame_data_id(
    extract::Path(database_id): extract::Path<i32>,
    extract::State(state): extract::State<Arc<AppState>>,
) -> Response {
    let delete_results = FrameMetadata::delete_in_db(database_id, &state.db);

    match delete_results {
        Ok(_) => {
            return json!({"id": format!("{} deleted", database_id)})
                .to_string()
                .into_response()
        }
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                json!({"error": format!("{error:?}"), "example":EXAMPLE_DATA}).to_string(),
            )
                .into_response();
        }
    };
}

pub async fn put_frame_data_id(
    extract::Path(database_id): extract::Path<i32>,
    extract::State(state): extract::State<Arc<AppState>>,
    payload: String,
) -> Response {
    let extracted_frame_data = match extract_json_frame(payload) {
        Ok(mut value) => {
            value.id = database_id;
            value
        }
        Err(value) => return value,
    };

    let frame_results = extracted_frame_data.update_in_db(&state.db);

    match frame_results {
        Ok(_) => {
            return serde_json::to_string(&extracted_frame_data)
                .unwrap()
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
) -> Response {
    let extracted_frame_data = match extract_json_frame(payload) {
        Ok(value) => value,
        Err(value) => return value,
    };
    let frame_results = extracted_frame_data.insert_in_db(&state.db);
    match frame_results {
        Ok(stats) => return json!({"id": stats.id}).to_string().into_response(),
        Err(stats) => {
            return (
                StatusCode::BAD_REQUEST,
                json!({"error": stats.to_string()}).to_string(),
            )
                .into_response()
        }
    };
}
