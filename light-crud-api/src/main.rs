mod database;

mod lights;

mod config;
use std::sync::Arc;

use config::read_or_create_config;
use database::frame::Frame;

use tokio;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::Notify;

// Function to await the shutdown signal
async fn wait_for_shutdown(notify: Arc<Notify>) {
    notify.notified().await;
    println!("Shutdown signal received. Closing server...");
}

#[tokio::main]
async fn main() {
    let path = "config.toml";
    let config = read_or_create_config(path).unwrap();
    println!("{config:?}");

    let shutdown_notify = Arc::new(Notify::new());
    let shutdown_notify_clone = shutdown_notify.clone();
    let shutdown_notify_main_loop = shutdown_notify.clone();

    // Spawn a task to listen for a shutdown signal (e.g., Ctrl+C)
    tokio::spawn(async move {
        let mut interrupt = signal(SignalKind::interrupt()).unwrap();
        let mut terminate = signal(SignalKind::terminate()).unwrap();

        tokio::select! {
            _ = interrupt.recv() => println!("Received SIGINT, shutting down..."),
            _ = terminate.recv() => println!("Received SIGTERM, shutting down..."),
        }
        shutdown_notify_clone.notify_one();
    });

    if config.debug.enable_webserver {
        let app = database::initialize::setup(&config).await;

        let listener =
            tokio::net::TcpListener::bind(format!("{}:{}", config.web.interface, config.web.port))
                .await
                .unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .with_graceful_shutdown(wait_for_shutdown(shutdown_notify))
                .await
                .unwrap();
        });
    }
    println!("After webserver");
    if config.debug.enable_lights {
        let mut controller = lights::controller::setup(&config);
        let mut test_frame = Frame::new();
        test_frame.data = String::from("[16711680,255, 65280]");
        lights::controller::write_frame(&test_frame, &mut controller).await;
    }
    println!("After lights");

    // tokio::time::sleep(Duration::from_millis(50)).await;
    shutdown_notify_main_loop.notified().await;
    println!("Ending Program ... ")
}
