use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Router,
};
use std::{collections::HashMap, sync::Arc};

use futures::executor::block_on;
use serde::Serialize;
use serde_json::json;

use sqlx::{Pool, Sqlite};

use super::{
    frame::{DataFrame, Frame},
    frame_data::FrameMetadata,
    initialize::AppState,
};

use crate::command::ChangeLighting;

const _EXAMPLE_DATA: &str = r#"
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

// const GET_SQL_STATEMENT: &str = "SELECT id, name, speed FROM Frame_Metadata WHERE id = ? ";
// const _DELETE_SQL_STATEMENT: &str = "DELETE FROM Frame_Metadata WHERE id = ? LIMIT 1";
// const _UPDATE_SQL_STATEMENT: &str = "UPDATE Frame_Metadata SET name = ?, speed= ? WHERE id = ?";
// const _INSERT_SQL_STATEMENT: &str = "INSERT INTO Frame_Metadata (name, speed) Values(?, ?)";

#[derive(Clone, Debug, Serialize)]
pub struct Animation {
    pub id: i32,
    pub name: String,
    pub speed: f64,
    pub frames: Vec<DataFrame>,
}
#[allow(dead_code, unused_variables)]
impl Animation {
    pub fn new() -> Self {
        Animation {
            id: -1,
            name: String::from(""),
            speed: 24.0,
            frames: Vec::new(),
        }
    }
    pub fn new_with_single_frame(color: u32) -> Self {
        let single_frame = Frame::new_with_color(color, 250);
        Animation {
            id: -1,
            name: String::from(""),
            speed: 24.0,
            frames: vec![DataFrame::from(&single_frame)],
        }
    }

    pub fn get_from_db(id: i32, db: &Pool<Sqlite>) -> Result<Self, sqlx::Error> {
        let frame_meta = match block_on(
            sqlx::query_as::<_, FrameMetadata>(
                "SELECT id, name, speed FROM Frame_Metadata WHERE id = ? ",
            )
            .bind(id)
            .fetch_one(db),
        ) {
            Ok(frame_metadata) => frame_metadata,
            Err(error) => return Err(error),
        };

        let child_frames = Frame::get_all_of_parent(frame_meta.id, db);
        let mut ani = Animation::from(frame_meta);
        ani.frames = child_frames.iter().map(|e| DataFrame::from(e)).collect();

        return Ok(ani);
    }

    pub fn update_in_db(self: &Self, db: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        todo!();
        // let result = block_on(
        //     sqlx::query("UPDATE Frames SET parent_id = ?, frame_id= ?, data= ? WHERE id = ?")
        //         .bind(self.parent_id)
        //         .bind(self.frame_id)
        //         .bind(self.data.clone())
        //         .bind(self.id)
        //         .execute(db),
        // );
        // return match result {
        //     Ok(_) => Ok(()),
        //     Err(err) => Err(err),
        // };
    }

    pub fn insert_in_db(self: &Self, db: &Pool<Sqlite>) -> Result<Self, sqlx::Error> {
        todo!()
        // let result = block_on(
        //     sqlx::query("INSERT INTO Frames (parent_id, frame_id, data) Values(?, ?, ?)")
        //         .bind(self.parent_id)
        //         .bind(self.frame_id)
        //         .bind(self.data.clone())
        //         .execute(db),
        // );

        // return match result {
        //     Ok(value) => Ok({
        //         let mut new_frame = self.clone();
        //         new_frame.id = value.last_insert_rowid() as i32;
        //         new_frame
        //     }),
        //     Err(err) => Err(err),
        // };
    }

    pub fn delete_in_db(id: i32, db: &Pool<Sqlite>) -> Result<HashMap<&str, i32>, sqlx::Error> {
        let mut result = HashMap::new();
        result.entry("frames_deleted").or_insert(0);
        result.entry("frame_data_deleted").or_insert(0);
        let child_frames = Frame::get_all_of_parent(id, db);
        for child_frame in child_frames {
            match Frame::delete_in_db(child_frame.id, db) {
                Ok(_) => {
                    result.entry("frames_deleted").and_modify(|e| *e += 1);
                }
                Err(error) => {
                    return {
                        println!(
                            "{}",
                            json!({"hashmap": result, "error": error.to_string()}).to_string()
                        );
                        Err(error)
                    }
                }
            }
        }
        let parent_data_result = FrameMetadata::delete_in_db(id, db);

        match parent_data_result {
            Ok(_) => {
                result.entry("frame_data_deleted").and_modify(|e| *e += 1);
            }
            Err(error) => {
                return {
                    println!(
                        "{}",
                        json!({"hashmap": result, "error": error.to_string()}).to_string()
                    );
                    Err(error)
                }
            }
        };
        return Ok(result);
    }
}

impl From<FrameMetadata> for Animation {
    fn from(a: FrameMetadata) -> Self {
        Animation {
            id: a.id,
            name: a.name,
            speed: a.speed,
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
        .route("/set/brightness/:id", post(set_brightness))
        .route("/set/speed/:fps", post(set_fps))
        .with_state(state);

    index.insert("/animation", "GET,POST");
    index.insert("/animation/:id", "GET,DELETE");
    index.insert("/animation/set/brightness/:value", "POST");
    index.insert("/animation/set/speed/:value", "POST");
    return app;
}
#[allow(unused_variables)]
async fn post_animations(State(state): State<Arc<AppState>>, payload: String) -> Response {
    todo!()
}

async fn set_brightness(
    Path(brightness_value): Path<u8>,
    State(state): State<Arc<AppState>>,
) -> Response {
    state
        .send_to_lights
        .send(ChangeLighting::Brightness(brightness_value))
        .await
        .unwrap();
    return (
        StatusCode::OK,
        json!({"brightness":brightness_value}).to_string(),
    )
        .into_response();
}
async fn set_fps(
    Path(new_fps): Path<f64>,
    State(state): State<Arc<AppState>>,
) -> Response {
    state
        .send_to_lights
        .send(ChangeLighting::Speed(new_fps))
        .await
        .unwrap();
    return (
        StatusCode::OK,
        json!({"speed":new_fps}).to_string(),
    )
        .into_response();
}

// pub fn get_frame_data(id: i32, db: &Pool<Sqlite>) -> Option<FrameMetadata> {
//     let frame_results = block_on(
//         sqlx::query_as::<_, FrameMetadata>(GET_SQL_STATEMENT)
//             .bind(id)
//             .fetch_one(db),
//     );

//     match frame_results {
//         Ok(result) => return Some(result),
//         Err(_) => return None,
//     }
// }

#[allow(unused_variables)]
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

/// Starts an animation and returns basic data about the animation
///
/// # Arguments
/// * `frame_id` - Extracted from the path from /:id
/// * `state` - Shared with the function through the router
///
/// # Returns
///
/// Resonse Object. {"animation":{"frame_data": meta_frame,"frames":number_of_frames,}}
///
pub async fn get_animation_id(
    Path(frame_id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let ani = match Animation::get_from_db(frame_id, &state.db) {
        Ok(value) => value,
        Err(value) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": value.to_string()}).to_string(),
            )
                .into_response()
        }
    };

    state
        .send_to_lights
        .send(ChangeLighting::Animation(ani.clone()))
        .await
        .expect("Could not send data");

    return (StatusCode::OK, json!({"animation":ani}).to_string()).into_response();
}

pub async fn delete_animation_id(
    Path(frame_id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let delete_results = match Animation::delete_in_db(frame_id, &state.db) {
        Ok(value) => value,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": error.to_string()}).to_string(),
            )
                .into_response()
        }
    };

    return serde_json::to_string(&delete_results)
        .unwrap()
        .into_response();
}
