mod database;
mod lights;
mod thread_utils;

mod config;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use config::read_or_create_config;
// use database::frame::Frame;

use futures::executor::block_on;
use thread_utils::NotifyChecker;
use tokio;

// use tokio::sync::mpsc;
use tokio::sync::Notify;
use tokio::time::timeout;

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
            println!("Inside the tokio thread");
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
        let mut recver = config.receving_channel;
        // let _loop_handle = tokio::spawn(async move {
        println!("in the controller thread");

        let mut controller = lights::controller::setup();
        // let mut test_frame = Frame::new();
        let looping_flag = shutdown_notify_controller_loop.flag.clone();
        while !looping_flag.load(Ordering::Relaxed) {
            // println!("inside loop");
            // println!("{shutdown_notify_controller_loop:?}");
            match timeout(Duration::from_secs(10), recver.recv()).await {
                Err(_) => print!("."),
                Ok(value) => match value {
                    None => println!("Error on the frame receive"),
                    Some(frame) => lights::controller::write_frame(&frame, &mut controller),
                    // Some(frame) => println!("{frame:?}"),
                },
            }
            block_on(tokio::time::sleep(Duration::from_millis(20)));

            // println!("{shutdown_notify_controller_loop:?}");
            // test_frame.data = String::from("[16711680,255, 65280]");
        }
        // });
        println!("Stopping Controller Loop ...");
    } else {
        println!("Not Starting Lighting Controller");
    }

    // tokio::time::sleep(Duration::from_millis(50)).await;
    if !config.debug.enable_lights {
        println!("Press Ctrl + C to end the program.");
        wait_for_shutdown(shutdown_notify_main_loop.notify).await;
    }
    println!("Ending Program ... ")
}
