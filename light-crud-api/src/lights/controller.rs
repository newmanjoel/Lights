use std::time::Duration;

use futures::executor::block_on;
use rs_ws281x::ChannelBuilder;
use rs_ws281x::ControllerBuilder;
// use rs_ws281x::StripType;
use futures;

use super::converter;

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

pub fn write_frame(frame: &Frame, controller: &mut rs_ws281x::Controller) {
    let frame_data = frame.data_out();

    assert_eq!(frame_data.len(), controller.leds(0).len());

    // let mut zipped: Vec<u32, [u8;4]> = frame_data.iter().zip(controller.leds_mut(0).iter_mut()).collect();

    // for (led_color, led LedColor) in zipped{

    // }

    for led_color in frame_data.iter() {
        let bytes = converter::ByteRGB::from_u32(*led_color);
        println!("{bytes:?}");

        let leds = controller.leds_mut(0);
        for mut led in leds.iter_mut().map(|e| LedColor::new(e)) {
            led.0 = [bytes.red, bytes.green, bytes.blue, 0];
            // *led = [bytes.red, bytes.green, bytes.blue, 0];
        }
        controller.render().unwrap();
        block_on(tokio::time::sleep(Duration::from_millis(1000)));
    }
}
