use rs_ws281x::ControllerBuilder;
use rs_ws281x::ChannelBuilder;
use rs_ws281x::StripType;

pub fn setup() -> rs_ws281x::Controller{
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
                .strip_type(StripType::Ws2811Grb)
                .brightness(100) // default: 255
                .build(),
        )
        .build()
        .unwrap();

    let leds = controller.leds_mut(0);

    for led in leds {
        *led = [0, 0, 255, 0];
    }

    controller.render().unwrap();
    return controller;
}