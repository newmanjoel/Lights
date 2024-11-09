use axum::{routing::get, Router};
// use serde_json::json;

use std::{collections::HashMap, sync::Arc};

use std::path::Path;

mod database_stuff;
use database_stuff::{get_or_create_sqlite_database, AppState};

mod frame;

mod frame_data;

mod location;

mod config;
use config::read_or_create_config;

#[tokio::main]
async fn main() {
    let path = "config.toml";
    let config = match read_or_create_config(path) {
        Ok(config) => config,
        Err(e) => panic!("Error: {}", e),
    };
    println!("{config:?}");
    let mut index: HashMap<&'static str, &str> = HashMap::new();

    let filepath = Path::new(config.database.file_path.as_str());
    let pool = get_or_create_sqlite_database(filepath).await.unwrap();
    let state: Arc<AppState> = Arc::new(AppState { db: pool });
    let frame_routes = frame::router(&mut index, state.clone());
    let frame_data_routes = frame_data::router(&mut index, state.clone());
    let location_routes = location::router(&mut index, state.clone());

    let app: Router = Router::new()
        .route(
            "/",
            get(|| async move { return serde_json::to_string_pretty(&index).unwrap().to_string() }),
        )
        .nest("/frame", frame_routes)
        .nest("/frame_data", frame_data_routes)
        .nest("/location", location_routes);

    // .route("/location", post(post_frame_data))
    // .route("/location/:id", get(get_frame_data_id))
    // .route("/location/:id", put(put_frame_data_id))
    // .route("/location/:id", delete(delete_frame_data_id))
    // .route("/add_animation", post(post_add_animation))
    // .with_state(state);

    // app = frame::setup(app, &mut index);
    // app = app.with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.web.port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn curl() {
//         // curl -X POST http://localhost:3000/do_id/3 -H "Content-Type: application/json" -d '{"key1":[1,2,3.3]}'
//         // Setting data for id: 3 with payload of: Object {"key1": Array [Number(1), Number(2), Number(3.3)]}%
//         assert!(true);
//     }
// }
