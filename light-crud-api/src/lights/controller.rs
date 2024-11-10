use rs_ws281x::ChannelBuilder;
use rs_ws281x::ControllerBuilder;
use rs_ws281x::StripType;

use crate::database::frame::Frame;
use super::converter;

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
                .strip_type(StripType::Ws2811Rgb)
                .brightness(100) // default: 255
                .build(),
        )
        .build()
        .unwrap();

    let leds = controller.leds_mut(0);

    for led in leds.into_iter() {
        *led = [0, 0, 255, 0];
    }

    controller.render().unwrap();
    let leds = controller.leds_mut(0);
    println!("led.len()={}", leds.len());
    return controller;
}

pub fn write_frame(frame: &Frame, controller: &mut rs_ws281x::Controller) {
    let frame_data = frame.data_out();

    let leds = controller.leds_mut(0);

    for led in leds.into_iter() {
        *led = [0, 0, 255, 0];
    }

    for led_color in frame_data.iter(){
        let bytes =converter::ByteRGB::from_u32(*led_color);
        println!("{bytes:?}");
    }

}
