use axum::{
    extract::{State,Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Router,
};
use std::{collections::HashMap, sync::Arc};

use serde::Serialize;
use serde_json::{json, Value};
use futures::executor::block_on;


use sqlx::{FromRow, Pool, Sqlite};

use super::{frame::Frame, frame_data::FrameMetadata, initialize::AppState};

const EXAMPLE_DATA: &str = r#"
{
    "animation":{
        "frame_data":{"name":"Some String Name","speed":24.0},
        "frames:[
            {"frame":{"frame_id":1, "data":"[1,2,3]"}},
            {"frame":{"frame_id":2, "data":"[1,2,3]"}},
            {"frame":{"frame_id":3, "data":"[1,2,3]"}},
            ...
        ]
    }
}
"#;
const GET_SQL_STATEMENT: &str = "SELECT id, name, speed FROM Frame_Metadata WHERE id = ? LIMIT 1";
const DELETE_SQL_STATEMENT: &str = "DELETE FROM Frame_Metadata WHERE id = ? LIMIT 1";
const UPDATE_SQL_STATEMENT: &str = "UPDATE Frame_Metadata SET name = ?, speed= ? WHERE id = ?";
const INSERT_SQL_STATEMENT: &str = "INSERT INTO Frame_Metadata (name, speed) Values(?, ?)";


#[derive(Clone, Debug, Serialize)]
pub struct Animation {
    pub id: i32,
    pub name: String,
    pub speed: f64,
    pub frames: Vec<Frame>,
}

impl Animation{
    pub fn new() -> Self{
        Animation{
            id: -1,
            name: String::from(""),
            speed: 24.0,
            frames: Vec::new(),
        }
    }
    pub fn new_with_single_frame(color: u32) -> Self{
        let single_frame = Frame::new_with_color(color, 250);
        Animation{
            id: -1,
            name: String::from(""),
            speed: 24.0,
            frames: vec![single_frame],
        }

    }
}

impl From<FrameMetadata> for Animation {
    fn from(a: FrameMetadata) -> Self {
        Animation {
            id: a.id,
            name: a.name,
            speed:a.speed,
            frames: Vec::new(),
        }
    }
}

pub fn router(index: &mut HashMap<&'static str, &str>, state: Arc<AppState>) -> Router {
    let app = Router::new()
        .route("/", post(post_animations))
        .route("/", get(get_animations))
        .route("/:id", get(get_animation_id))
        .route("/:id", delete(delete_animation_id))
        .with_state(state);

    index.insert("/frame_data", "GET,POST");
    index.insert("/frame_data/:id", "GET,DELETE");
    return app;
}

async fn post_animations(State(state): State<Arc<AppState>>,
    payload: String,
) -> Response {
    todo!()
}

pub fn get_frame_data(id: i32, db: &Pool<Sqlite>) -> Option<FrameMetadata>{
    let frame_results = block_on(
        sqlx::query_as::<_, FrameMetadata>(GET_SQL_STATEMENT)
        .bind(id)
        .fetch_one(db));

    match frame_results{
        Ok(result) => return Some(result),
        Err(_) => return None,
    }
}

pub fn get_all_frames_of_parent(id: i32, db: &Pool<Sqlite>) -> Vec<Frame>{
    let frame_results = block_on(
        sqlx::query(
        "SELECT id, parent_id, frame_id, data FROM Frames WHERE parent_id = ?")
        .bind(id)
        .fetch_all(db));
    
    let sqlite_rows = match frame_results{
        Err(_) => return Vec::new(),
        Ok(value) => value,
    };

    let frames: Vec<Frame> = sqlite_rows.iter().map(|e| Frame::from_row(e).unwrap()).collect();
    return frames;
}

async fn get_animations(State(state): State<Arc<AppState>>) -> Response {
    todo!();

    // match frame_results {
    //     Ok(value) => return serde_json::to_string(&value).unwrap().into_response(),
    //     Err(value) => {
    //         return (
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             json!({"error": value.to_string()}).to_string(),
    //         )
    //             .into_response()
    //     }
    // };
}

pub async fn get_animation_id(
    Path(frame_id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let meta_frame = match get_frame_data(frame_id, &state.db){
        Some(result) => result,
        None => return (StatusCode::BAD_REQUEST, json!({"error":"parent_id not found"}).to_string()).into_response(),
    };

    let frames = get_all_frames_of_parent(meta_frame.id, &state.db);

    return (StatusCode::OK, json!({
        "animation":{
            "frame_data": meta_frame,
            "frames":frames,
        }
    }).to_string()).into_response();
    
}

pub async fn delete_animation_id(
    Path(frame_id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let frame_results = sqlx::query_as::<_, FrameMetadata>(GET_SQL_STATEMENT)
        .bind(frame_id)
        .fetch_one(&state.db)
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