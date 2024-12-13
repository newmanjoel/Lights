mod command;
mod config;
mod database;
mod lights;
mod thread_utils;

use config::read_or_create_config;

use thread_utils::NotifyChecker;

use tokio;
use tokio::sync::Notify;


use std::sync::Arc;

// Function to await the shutdown signal
async fn wait_for_shutdown(notify: Arc<Notify>) {
    notify.notified().await;
    println!("wait_for_shutdown: Shutdown signal received. Closing server...");
}

#[tokio::main]
async fn main() {
    let path = "config.toml";
    let config = read_or_create_config(path).unwrap();
    println!("{config:?}\n\n");

    let notifier = NotifyChecker::new();

    // Spawn a task to listen for a shutdown signal (e.g., Ctrl+C)
    let shutdown_signal_notifier = notifier.clone();
    tokio::spawn(thread_utils::wait_for_signals(shutdown_signal_notifier));

    let mut threads = Vec::new();

    if config.debug.enable_timed_brightness {
        let timed_brightness_notifier = notifier.clone();
        // let brightness_tx = config.brightness_comms.sending_channel.clone();
        let command_comms_tx = config.command_comms.sending_channel.clone();
        threads.push(tokio::spawn(thread_utils::timed_brightness(
            command_comms_tx,
            timed_brightness_notifier,
        )));
    }

    if config.debug.enable_webserver {
        let shutdown_notify_web_server = notifier.clone();

        let app = database::initialize::setup(&config).await;

        let listener =
            tokio::net::TcpListener::bind(format!("{}:{}", config.web.interface, config.web.port))
                .await
                .unwrap();
        let handle = tokio::spawn(async move {
            println!("{}", "Webserver: Starting");
            axum::serve(listener, app.into_make_service())
                .with_graceful_shutdown(wait_for_shutdown(shutdown_notify_web_server.notify))
                .await
                .unwrap();
            println!("{}", "Webserver: Stopped");
        });
        threads.push(handle);
        // .await
        // .expect("Webserver errored out");
        println!("Webserver: Started");
    } else {
        println!("Controller: N/A");
    }
    if config.debug.enable_lights {
        let light_shutdown_notifier = notifier.clone();
        // let animation_comms_rx = config.animation_comms.receving_channel;
        // let brightness_comms_rx = config.brightness_comms.receving_channel;
        let command_comms_rx = config.command_comms.receving_channel;
        let current_data = config.current_data.clone();
        use lights::controller::light_loop;

        light_loop(
            light_shutdown_notifier,
            command_comms_rx,
            current_data,
        )
        .await;

        // threads.push(handle);
    } else {
        println!("Not Starting Lighting Controller");
    }

    if !config.debug.enable_lights {
        let shutdown_notify_main_loop = notifier.clone();
        println!("Press Ctrl + C to end the program.");
        wait_for_shutdown(shutdown_notify_main_loop.notify).await;
    }
    for thread in threads {
        println!("{thread:?}");
        if thread.is_finished() {
            println!("Finished: {thread:?}");
        }
        thread.await.unwrap();
    }
    println!("Ending Program ... ")
}
