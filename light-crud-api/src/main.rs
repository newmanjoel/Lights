mod database;
mod lights;
mod thread_utils;

mod config;
use std::sync::Arc;

use config::read_or_create_config;
use database::frame::Frame;

use thread_utils::NotifyChecker;
use tokio;

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

    let notifier = NotifyChecker::new();
    let shutdown_signal_notifier = notifier.clone();
    let shutdown_notify_web_server = notifier.clone();
    let shutdown_notify_main_loop = notifier.clone();
    let shutdown_notify_controller_loop = notifier.clone();

    // Spawn a task to listen for a shutdown signal (e.g., Ctrl+C)
    tokio::spawn(thread_utils::wait_for_signals(shutdown_signal_notifier));

    if config.debug.enable_webserver {
        println!("Starting Webserver ... ");
        let app = database::initialize::setup(&config).await;

        let listener =
            tokio::net::TcpListener::bind(format!("{}:{}", config.web.interface, config.web.port))
                .await
                .unwrap();
        let _handle = tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(wait_for_shutdown(shutdown_notify_web_server.notify))
            .await
            .unwrap();
        });
        // .await
        // .expect("Webserver errored out");
        println!("Started Webserver ... ");
    } else {
        println!("Not Starting Webserver");
    }
    if config.debug.enable_lights {
        println!("Starting Controller loop ... ");
        let _loop_handle = tokio::spawn(async move {
            println!("in the controller thread");
            let mut controller = lights::controller::setup();
            let mut test_frame = Frame::new();
            while !shutdown_notify_controller_loop.is_notified() {
                println!("inside loop");
                test_frame.data = String::from("[16711680,255, 65280]");
                lights::controller::write_frame(&test_frame, &mut controller);
            }
        });
        println!("Started Controller Loop ...");
    } else {
        println!("Not Starting Lighting Controller");
    }

    // tokio::time::sleep(Duration::from_millis(50)).await;
    wait_for_shutdown(shutdown_notify_main_loop.notify).await;
    println!("Ending Program ... ")
}
