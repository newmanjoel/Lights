use axum::{
    extract,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Router,
};
use futures::executor::block_on;
use std::{any::type_name, collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use sqlx::{FromRow, Pool, Sqlite};

use crate::command::ChangeLighting;
use crate::database::initialize::AppState;

use super::{animation, frame_data::FrameMetadata};

const EXAMPLE_DATA: &str = r#"{"frame":{"parent_id":1,"frame_id":1, "data":"[1,2,3]"}}"#;

#[derive(Clone, FromRow, Debug, Serialize, Deserialize)]
pub struct Frame {
    pub id: i32,
    pub parent_id: i64,
    pub frame_id: i64,
    pub data: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataFrame {
    pub id: i32,
    pub parent_id: i64,
    pub frame_id: i64,
    pub data: Vec<u32>,
}

impl From<&Frame> for DataFrame {
    fn from(a: &Frame) -> Self {
        DataFrame {
            id: a.id,
            parent_id: a.parent_id,
            frame_id: a.frame_id,
            data: a.data_out(),
        }
    }
}
impl From<&DataFrame> for Frame {
    fn from(a: &DataFrame) -> Self {
        Frame {
            id: a.id,
            parent_id: a.parent_id,
            frame_id: a.frame_id,
            data: json!(a.data).to_string(),
        }
    }
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
            // need to parse on value data type

            Some(value) => match value.as_str() {
                Some(value) => value,
                None => {
                    return Err(json!({
                        "error":format!("could not convert data entry to a str"), 
                        "debug":format!("passed in value was of type: {}","serde_json::value::Value") 
                    }))
                }
            },
            None => return Err(json!({"error":format!("could not find data")})),
        };

        return Ok(Self {
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

    pub fn get_from_db(id: i32, db: &Pool<Sqlite>) -> Result<Self, sqlx::Error> {
        let result = block_on(
            sqlx::query_as::<_, Self>(
                "SELECT id, parent_id, frame_id, data FROM Frames WHERE id = ?",
            )
            .bind(id)
            .fetch_one(db),
        );
        return result;
    }

    pub fn update_in_db(self: &Self, db: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        let result = block_on(
            sqlx::query("UPDATE Frames SET parent_id = ?, frame_id= ?, data= ? WHERE id = ?")
                .bind(self.parent_id)
                .bind(self.frame_id)
                .bind(self.data.clone())
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
            sqlx::query("INSERT INTO Frames (parent_id, frame_id, data) Values(?, ?, ?)")
                .bind(self.parent_id)
                .bind(self.frame_id)
                .bind(self.data.clone())
                .execute(db),
        );

        return match result {
            Ok(value) => Ok({
                let mut new_frame = self.clone();
                new_frame.id = value.last_insert_rowid() as i32;
                new_frame
            }),
            Err(err) => Err(err),
        };
    }

    pub fn delete_in_db(id: i32, db: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        let result = block_on(
            sqlx::query("DELETE FROM Frames WHERE id = ?")
                .bind(id)
                .execute(db),
        );

        return match result {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        };
    }

    pub fn get_all_from_db(db: &Pool<Sqlite>) -> Result<Vec<Self>, sqlx::Error> {
        let frame_results = block_on(
            sqlx::query_as::<_, Self>("SELECT id, parent_id, frame_id, data FROM Frames")
                .fetch_all(db),
        );
        return frame_results;
    }

    pub fn get_all_of_parent(parent_id: i32, db: &Pool<Sqlite>) -> Vec<Self> {
        let frame_results = block_on(
            sqlx::query("SELECT id, parent_id, frame_id, data FROM Frames WHERE parent_id = ?")
                .bind(parent_id)
                .fetch_all(db),
        );
        match frame_results {
            Err(_) => return Vec::new(),
            Ok(value) => return value.iter().map(|e| Frame::from_row(e).unwrap()).collect(),
        };
    }
}


fn type_of<T>(_: &T) -> &'static str{
    return type_name::<T>();
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

fn extract_json_frame(payload: String) -> Result<Frame, Response> {
    let json_payload: Value = match serde_json::from_str(&payload) {
        Ok(result) => result,
        Err(error) => {
            return Err((
                StatusCode::BAD_REQUEST,
                json!({"error":format!("{error:?}"), "example":EXAMPLE_DATA}).to_string(),
            )
                .into_response())
        }
    };

    let frame_dict = match json_payload.get("frame") {
        Some(value) => value,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                json!({"error":"frame_data not found", "example":EXAMPLE_DATA}).to_string(),
            )
                .into_response())
        }
    };

    match Frame::extract_from_dict(frame_dict) {
        Ok(value) => return Ok(value),
        Err(value) => return Err((StatusCode::BAD_REQUEST, value.to_string()).into_response()),
    };
}

pub async fn get_frame_id(
    extract::Path(id): extract::Path<i32>,
    extract::State(state): extract::State<Arc<AppState>>,
) -> Response {
    let frame_results = Frame::get_from_db(id, &state.db);

    match frame_results {
        Ok(value) => return serde_json::to_string(&value).unwrap().into_response(),
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                json!({"error": format!("{error:?}"), "example":EXAMPLE_DATA}).to_string(),
            )
                .into_response()
        }
    };
}

pub async fn show_frame_id(
    extract::Path(frame_id): extract::Path<i32>,
    extract::State(state): extract::State<Arc<AppState>>,
) -> Response {
    let frame_results = Frame::get_from_db(frame_id, &state.db);

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
    let meta_frame = FrameMetadata::get_from_db(data.parent_id as i32, &state.db).unwrap();

    let mut ani = animation::Animation::from(meta_frame);
    ani.frames.push(DataFrame::from(&data));

    state
        .send_to_lights
        .send(ChangeLighting::Animation(ani))
        .await
        .expect("Could not send data");

    return serde_json::to_string(&data).unwrap().into_response();
}

pub async fn get_all_frame(extract::State(state): extract::State<Arc<AppState>>) -> Response {
    let frame_results = Frame::get_all_from_db(&state.db);

    match frame_results {
        Ok(value) => return serde_json::to_string(&value).unwrap().into_response(),
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": format!("{error:?}"), "example":EXAMPLE_DATA}).to_string(),
            )
                .into_response()
        }
    };
}

pub async fn post_frame(
    extract::State(state): extract::State<Arc<AppState>>,
    payload: String,
) -> Response {
    let extraction_results = extract_json_frame(payload);

    let frame = match extraction_results {
        Ok(value) => value,
        Err(response) => return response,
    };

    let insert_results = frame.insert_in_db(&state.db);

    match insert_results {
        Ok(stats) => return json!({"id": stats.id}).to_string().into_response(),
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
    extract::Path(database_id): extract::Path<i32>,
    extract::State(state): extract::State<Arc<AppState>>,
    payload: String,
) -> Response {
    let extraction_results = extract_json_frame(payload);

    let frame = match extraction_results {
        Ok(mut value) => {
            value.id = database_id;
            value
        }
        Err(response) => return response,
    };

    let update_results = frame.update_in_db(&state.db);

    match update_results {
        Ok(_) => return serde_json::to_string(&frame).unwrap().into_response(),
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
    extract::Path(database_id): extract::Path<i32>,
    extract::State(state): extract::State<Arc<AppState>>,
) -> Response {
    let delete_results = Frame::delete_in_db(database_id, &state.db);

    match delete_results {
        Ok(_) => {
            return json!({"id": format!("row {} deleted", database_id)})
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
