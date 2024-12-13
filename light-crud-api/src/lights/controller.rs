// use std::time::Duration;

use std::io;
use std::io::Write;
use std::time::Duration;

use colored::Colorize;
use futures::executor::block_on;
// use futures::executor::block_on;
use rs_ws281x::ChannelBuilder;
use rs_ws281x::ControllerBuilder;
use tokio::time::timeout;
// use rs_ws281x::StripType;
// use futures;

use super::converter;

use crate::config::Config;
use crate::database::animation::Animation;
use crate::database::frame::DataFrame;
use crate::thread_utils::NotifyChecker;

const FRONT_OF_HOUSE_CHANNEL: usize = 1;
const FRONT_OF_HOUSE_PIN: i32 = 19;

const FRONT_ENTRYWAY_CHANNEL: usize = 0;
const FRONT_ENTRYWAY_PIN: i32 = 12;

pub fn setup() -> rs_ws281x::Controller {
    // Construct a single channel controller. Note that the
    // Controller is initialized by default and is cleaned up on drop

    let mut controller: rs_ws281x::Controller = ControllerBuilder::new()
        .freq(800_000)
        .dma(10)
        .channel(
            FRONT_ENTRYWAY_CHANNEL, // Channel Index
            ChannelBuilder::new()
                .pin(FRONT_ENTRYWAY_PIN) // GPIO 12 = PWM0 // Default was 10
                .count(250) // Number of LEDs
                .strip_type(rs_ws281x::StripType::Ws2811Bgr)
                .brightness(100) // default: 255
                .build(),
        )
        .channel(
            FRONT_OF_HOUSE_CHANNEL, // Channel Index
            ChannelBuilder::new()
                .pin(FRONT_OF_HOUSE_PIN) // GPIO 12 = PWM0 // Default was 10
                .count(250) // Number of LEDs
                .strip_type(rs_ws281x::StripType::Ws2811Bgr)
                .brightness(100) // default: 255
                .build(),
        )
        .build()
        .unwrap();

    let leds = controller.leds_mut(0);

    // thinking the format is Red, Green, Blue
    for led in leds.into_iter() {
        *led = [0, 255, 0, 0];
    }
    controller.render().unwrap();
    return controller;
}

struct LedColor([u8; 4]);

#[allow(dead_code)]
impl LedColor {
    fn new(value: &mut [u8; 4]) -> Self {
        return LedColor(*value);
    }

    fn red(&self) -> u8 {
        return self.0[0];
    }
}

#[allow(unused_mut)]
pub fn write_frame(frame: &DataFrame, controller: &mut rs_ws281x::Controller) {
    // println!("write_frame: top");
    for (led_color, mut led) in frame.data.iter().zip(controller.leds_mut(FRONT_OF_HOUSE_CHANNEL).iter_mut()) {
        let bytes = converter::ByteRGB::from_u32(*led_color);
        *led = [bytes.red, bytes.green, bytes.blue, 0];
    }
    for (led_color, mut led) in frame.data.iter().zip(controller.leds_mut(FRONT_ENTRYWAY_CHANNEL).iter_mut()) {
        let bytes = converter::ByteRGB::from_u32(*led_color);
        *led = [bytes.red, bytes.green, bytes.blue, 0];
    }
    controller.render().unwrap();
    // println!("write_frame: bottom");
}

pub async fn light_loop(
    shutdown_notifier: NotifyChecker,
    mut animation_receiver: tokio::sync::mpsc::Receiver<Animation>,
    mut brightness_receiver: tokio::sync::mpsc::Receiver<u8>,
) -> () {
    println!("Controller: Starting");
    // let shutdown_notify_controller_loop = notifier.clone();
    // let mut animation_receiver = config.animation_comms.receving_channel;
    // let mut brightness_receiver = config.brightness_comms.receving_channel;

    let mut controller = setup();
    // let looping_flag = shutdown_notifier.flag.clone();

    let mut working_animation = Animation::new_with_single_frame(255);
    working_animation.speed = 1.5;
    let mut working_index = 0;
    let mut working_frame_size = 1;
    let mut working_time = (1000.0 / working_animation.speed) as u64;
    while !shutdown_notifier.is_notified() {
        // println!("top: {}", shutdown_notifier.is_notified());
        // if there is a new animation, load it and set the relevant counters
        match timeout(Duration::from_micros(1), animation_receiver.recv()).await {
            Err(err) => {
                // println!("animation: {err}");
            }
            Ok(value) => match value {
                None => println!("Error on the animation receive"),
                Some(frame) => {
                    working_animation = frame;
                    working_index = 0;
                    working_frame_size = working_animation.frames.len();
                    working_time = (1000.0 / working_animation.speed) as u64;
                    println!("setting the loop time to {working_time:?}ms for {} fps", working_animation.speed);
                }
            },
        }
        match timeout(Duration::from_micros(1), brightness_receiver.recv()).await {
            Err(err) => {
                // println!("brightness: {err}");
            }
            Ok(value) => match value {
                None => println!("Error on the animation receive"),
                Some(brightness_value) => {
                    controller.set_brightness(0, brightness_value);
                    controller.set_brightness(1, brightness_value);
                    println!("Setting the Brightness to {}", brightness_value);
                }
            },
        }

        let working_frame = &working_animation.frames[working_index];
        working_index += 1;
        working_index = working_index % working_frame_size;
        write_frame(working_frame, &mut controller);
        std::thread::sleep(Duration::from_millis(working_time));
        // tokio::time::sleep(Duration::from_millis(working_time)).await;
        // println!("bottom: {}", shutdown_notifier.is_notified());
    }
    println!("{}", "Controller: Stopping".red());
}

// /home/pi/Lights/db/sqlite.db
