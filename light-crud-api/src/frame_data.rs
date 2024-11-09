use axum::{extract};
use std::sync::Arc;

use serde::Serialize;
use serde_json::{Result, Value};
use sqlx::FromRow;


use crate::database_stuff::{AppState};
use crate::frame::Frame;


#[derive(Clone, FromRow, Debug, Serialize)]
pub struct Frame_Metadata {
    pub id: i32,
    pub name: String,
    pub speed: f32,
}


pub async fn get_frame_data_id(
    extract::Path(frame_id): extract::Path<i32>,
    extract::State(state): extract::State<Arc<AppState>>,
) -> String {

    let frame_results = sqlx::query_as::<_, Frame_Metadata>("SELECT id, name, speed FROM Frame_Metadata WHERE id = ?")
        .bind(frame_id)
        .fetch_all(&state.db)
        .await
        .unwrap();


    println!("do_something: {frame_results:?}");
    let data: Vec<String> = frame_results
        .iter()
        .map(|frame_metadata| serde_json::to_string(&frame_metadata).unwrap())
        .to_owned()
        .collect();
    let owned_data = data.join("|") + &frame_id.to_string();
    return owned_data;
}

pub async fn post_frame_data_id(
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