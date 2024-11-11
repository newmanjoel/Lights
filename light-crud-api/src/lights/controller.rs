// use std::time::Duration;

// use futures::executor::block_on;
use rs_ws281x::ChannelBuilder;
use rs_ws281x::ControllerBuilder;
// use rs_ws281x::StripType;
// use futures;

use super::converter;

use crate::database::animation::Animation;
use crate::database::frame::Frame;

pub fn setup() -> rs_ws281x::Controller {
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
pub fn write_frame(frame: &Frame, controller: &mut rs_ws281x::Controller) {
    let mut frame_data = frame.data_out();
    // let mut led_channel = controller.leds(0);
    // let led_num = led_channel.len();
    // let data_len = frame_data.len();
    // println!("\nframe_data length {data_len:?}");
    // println!("led_controller length {led_num:?}");
    // assert_eq!(data_len, led_num);
    println!("assuming that they are the same size ... ");

    // let zipped = frame_data
    //     .iter_mut()
    //     .zip(controller.leds_mut(0).iter_mut().map(|e| LedColor::new(e)));

    // for (led_color, mut led) in zipped {
    //     let bytes = converter::ByteRGB::from_u32(*led_color);
    //     led.0 = [bytes.red, bytes.blue, bytes.green, 0]
    // }
    // controller.render().unwrap();
    // println!("wrote the frame to the lights");
    // block_on(tokio::time::sleep(Duration::from_millis(1000)));

    for (led_color, mut led) in frame_data.iter().zip(controller.leds_mut(0).iter_mut()) {
        let bytes = converter::ByteRGB::from_u32(*led_color);
        // println!("{bytes:?}");
        *led = [bytes.red, bytes.green, bytes.blue, 0];

        // // let leds = controller.leds_mut(0);
        // for mut led in leds.iter_mut(){
        //     // led.0 = [bytes.red, bytes.green, bytes.blue, 0];
        // }
    }
    controller.render().unwrap();
}

// /home/pi/Lights/db/sqlite.db