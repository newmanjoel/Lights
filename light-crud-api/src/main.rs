use axum::{routing::get, Router};
use std::path::Path;
use std::{collections::HashMap, sync::Arc};

mod database;
use crate::database::initialize::{get_or_create_sqlite_database, AppState};
use crate::database::{frame, frame_data, location};

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
