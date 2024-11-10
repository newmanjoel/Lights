mod database;

mod config;
use config::read_or_create_config;

#[tokio::main]
async fn main() {
    let path = "config.toml";
    let config = read_or_create_config(path).unwrap();
    println!("{config:?}");

    let app = database::initialize::setup(&config).await;

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.web.interface, config.web.port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
