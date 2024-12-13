use std::sync::{Arc, Mutex};

use chrono::{Local, Timelike};
use std::time::Duration;

use crate::{command::ChangeLighting, config::DayNightConfig, thread_utils::NotifyChecker};

struct TimedBrightness{
    hour:u32,
    brightness:u8
}

fn get_day_night(time_config: &Arc<Mutex<DayNightConfig>>) -> (TimedBrightness,TimedBrightness){
    let day = TimedBrightness{
        hour:{
            let config = time_config.lock().unwrap();
            config.day_hour
        },
        brightness:{
            let config = time_config.lock().unwrap();
            config.day_brightness
        }
    };
    let night = TimedBrightness{
        hour:{
            let config = time_config.lock().unwrap();
            config.night_hour
        },
        brightness:{
            let config = time_config.lock().unwrap();
            config.night_brightness
        }
    };
    
    return (day, night);
}



pub async fn timed_brightness(sender: tokio::sync::mpsc::Sender<ChangeLighting>, shutdown: NotifyChecker, time_config: Arc<Mutex<DayNightConfig>>) {
    // let night_brightness: u8 = 100;
    // let day_brightness: u8 = 1;
    println!("Timed Brightness: Starting");
    while !shutdown.is_notified() {
        let now = Local::now();

        let (day, night) = get_day_night(&time_config);
        
        if now.time().hour() > night.hour {
            sender.send(ChangeLighting::Brightness(night.brightness)).await.unwrap();
            // sender.send(night_brightness).await.unwrap();
        } else if now.time().hour() > day.hour {
            sender.send(ChangeLighting::Brightness(day.brightness)).await.unwrap();
            // sender.send(day_brightness).await.unwrap();
        }
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
    println!("Timed Brightness: Stopped");
}
