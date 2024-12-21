use std::sync::{Arc, Mutex};
use tokio::time::{self, Duration};

use chrono::{Local, Timelike};
// use std::time::Duration;

use crate::config::CurrentAnimationData;
use crate::thread_utils::Notifier;
use crate::{command::ChangeLighting, config::DayNightConfig};

struct TimedBrightness {
    hour: u32,
    brightness: u8,
}

fn get_day_night(time_config: &Arc<Mutex<DayNightConfig>>) -> (TimedBrightness, TimedBrightness) {
    let day = TimedBrightness {
        hour: {
            let config = time_config.lock().unwrap();
            config.day_hour
        },
        brightness: {
            let config = time_config.lock().unwrap();
            config.day_brightness
        },
    };
    let night = TimedBrightness {
        hour: {
            let config = time_config.lock().unwrap();
            config.night_hour
        },
        brightness: {
            let config = time_config.lock().unwrap();
            config.night_brightness
        },
    };

    return (day, night);
}

pub async fn timed_brightness(
    sender: tokio::sync::mpsc::Sender<ChangeLighting>,
    mut shutdown: Notifier<bool>,
    time_config: Arc<Mutex<DayNightConfig>>,
    current_data: CurrentAnimationData,
) {
    // let night_brightness: u8 = 100;
    // let day_brightness: u8 = 1;
    println!("Timed Brightness: Starting");
    let mut desired_brightness: Option<u8> = None;

    while !*shutdown.receving_channel.borrow_and_update() {
        let timeout = time::sleep(Duration::from_secs(10));

        tokio::select! {
            _ = timeout=> {
                let now = Local::now();
                let (day, night) = get_day_night(&time_config);
                // default night.hour is 15 (or 3 pm )
                // default day.hour is 6 (or 6 am)
                if now.time().hour() > night.hour {
                    desired_brightness = Some(night.brightness);
                } else if now.time().hour() > day.hour {
                    desired_brightness = Some(day.brightness);
                }
                match desired_brightness {
                    Some(brightness) => {
                        let current_brightness = *current_data.brightness.receving_channel.borrow();
                        let enabled= time_config.lock().unwrap().enabled;
                        if brightness != current_brightness && enabled {
                            sender
                                .send(ChangeLighting::Brightness(brightness))
                                .await
                                .unwrap();
                        }
                    }
                    None => {}
                }
                desired_brightness = None;
                }

            _ = shutdown.receving_channel.changed() =>{
                println!("Shutdown signal changed");
            }

        }
    }
    println!("Timed Brightness: Stopped");
}
