mod database;
mod lights;
mod thread_utils;

mod config;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use config::read_or_create_config;
// use database::frame::Frame;

use database::animation::Animation;
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
    
    
    
    
    
    // Spawn a task to listen for a shutdown signal (e.g., Ctrl+C)
    let shutdown_signal_notifier = notifier.clone();
    tokio::spawn(thread_utils::wait_for_signals(shutdown_signal_notifier));

    if config.debug.enable_timed_brightness{
        let timed_brightness_notifier = notifier.clone();
        let brightness_tx = config.brightness_comms.sending_channel.clone();
        tokio::spawn(thread_utils::timed_brightness(brightness_tx , timed_brightness_notifier));
    }

    if config.debug.enable_webserver {
        println!("Starting Webserver ... ");
        let shutdown_notify_web_server = notifier.clone();

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
        let shutdown_notify_controller_loop = notifier.clone();
        let mut animation_receiver = config.animation_comms.receving_channel;
        let mut brightness_receiver = config.brightness_comms.receving_channel;
        println!("in the controller thread");

        let mut controller = lights::controller::setup();
        let looping_flag = shutdown_notify_controller_loop.flag.clone();

        let mut working_animation = Animation::new_with_single_frame(255);
        let mut working_index = 0;
        let mut working_frame_size = 1;
        let mut working_time = 20;
        while !looping_flag.load(Ordering::Relaxed) {
            // if there is a new animation, load it and set the relevant counters
            match timeout(Duration::from_micros(1), animation_receiver.recv()).await {
                Err(_) => {}
                Ok(value) => match value {
                    None => println!("Error on the animation receive"),
                    Some(frame) => {
                        working_animation = frame;
                        working_index = 0;
                        working_frame_size = working_animation.frames.len();
                        working_time = (1000.0 / working_animation.speed) as u64;
                        println!("{working_time:?}ms vs {}", working_animation.speed);
                    }
                },
            }
            match timeout(Duration::from_micros(1), brightness_receiver.recv()).await {
                Err(_) => {}
                Ok(value) => match value {
                    None => println!("Error on the animation receive"),
                    Some(brightness_value) => {
                        controller.set_brightness(0, brightness_value);
                        println!("Setting the Brightness to {}", brightness_value);
                    }
                },
            }

            let working_frame = &working_animation.frames[working_index];
            working_index += 1;
            working_index = working_index % working_frame_size;
            lights::controller::write_frame(working_frame, &mut controller);
            block_on(tokio::time::sleep(Duration::from_millis(working_time)));
        }
        println!("Stopping Controller Loop ...");
    } else {
        println!("Not Starting Lighting Controller");
    }

    if !config.debug.enable_lights {
        let shutdown_notify_main_loop = notifier.clone();
        println!("Press Ctrl + C to end the program.");
        wait_for_shutdown(shutdown_notify_main_loop.notify).await;
    }
    println!("Ending Program ... ")
}
