use std::time::Duration;

use rs_ws281x::ChannelBuilder;
use rs_ws281x::ControllerBuilder;
// use rs_ws281x::StripType;

use super::converter;
use crate::config::Config;
use crate::database::frame::Frame;

pub fn setup(config: &Config) -> rs_ws281x::Controller {
    // Construct a single channel controller. Note that the
    // Controller is initialized by default and is cleaned up on drop

    let mut controller: rs_ws281x::Controller = ControllerBuilder::new()
        .freq(800_000)
        .dma(10)
        .channel(
            0, // Channel Index
            ChannelBuilder::new()
                .pin(12) // GPIO 12 = PWM0 // Default was 10
                .count(250) // Number of LEDs
                .strip_type(config.debug.strip_type)
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

pub async fn write_frame(frame: &Frame, controller: &mut rs_ws281x::Controller) {
    let frame_data = frame.data_out();

    for led_color in frame_data.iter() {
        let bytes = converter::ByteRGB::from_u32(*led_color);
        println!("{bytes:?}");

        let leds = controller.leds_mut(0);
        for led in leds.into_iter() {
            *led = [bytes.red, bytes.green, bytes.blue, 0];
        }
        controller.render().unwrap();
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}
