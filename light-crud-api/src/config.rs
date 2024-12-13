use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[allow(dead_code, unused_imports)]
use tokio::sync::mpsc::{channel, Receiver, Sender};
use toml;

use crate::command::ChangeLighting;
use crate::thread_utils::CompactSender;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TOMLConfig {
    pub database: DatabaseConfig,
    pub web: WebConfig,
    pub modules: LoadModuleConfig,
    pub day_night: DayNightConfig,
}

#[derive(Debug)]
pub struct Config {
    pub database: DatabaseConfig,
    pub web: WebConfig,
    pub module_enable: LoadModuleConfig,
    pub command_comms: CompactSender<ChangeLighting>,
    pub current_data: CurrentAnimationData,
    pub day_night: Arc<Mutex<DayNightConfig>>,
}
// file_path = "/home/pi/Lights/db/sqlite.db"

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CurrentAnimationData {
    pub brightness: Arc<Mutex<u8>>,
    pub animation_index: Arc<Mutex<i32>>,
    pub frame_index: Arc<Mutex<usize>>,
    pub animation_speed: Arc<Mutex<f64>>,
}

impl Default for CurrentAnimationData {
    fn default() -> Self {
        CurrentAnimationData {
            brightness: Arc::new(Mutex::new(100)),
            animation_index: Arc::new(Mutex::new(0)),
            frame_index: Arc::new(Mutex::new(0)),
            animation_speed: Arc::new(Mutex::new(24.0)),
        }
    }
}

impl CurrentAnimationData {
    pub fn to_json(&self) -> serde_json::Value {
        let brightness = self.brightness.lock().unwrap();
        let animation_index = self.animation_index.lock().unwrap();
        let frame_index = self.frame_index.lock().unwrap();
        let animation_speed = self.animation_speed.lock().unwrap();

        return json!({
            "brightness": *brightness,
            "animation_index": *animation_index,
            "frame_index": *frame_index,
            "animation_speed": *animation_speed,
        });
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatabaseConfig {
    pub file_path: String,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct LoadModuleConfig {
    pub on_raspberry_pi: bool,
    pub webserver: bool,
    pub lights: bool,
    pub timed_brightness: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DayNightConfig {
    pub day_hour: u32,
    pub day_brightness: u8,
    pub night_hour: u32,
    pub night_brightness: u8,
}

impl Default for DayNightConfig {
    fn default() -> Self {
        DayNightConfig {
            day_hour: 6,
            day_brightness: 1,
            night_hour: 16,
            night_brightness: 100,
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebConfig {
    pub port: i32,
    pub interface: String,
}

impl Default for WebConfig {
    fn default() -> Self {
        WebConfig {
            port: 3000,
            interface: "0.0.0.0".to_string(),
        }
    }
}
impl Default for LoadModuleConfig {
    fn default() -> Self {
        LoadModuleConfig {
            on_raspberry_pi: false,
            webserver: false,
            lights: false,
            timed_brightness: false,
        }
    }
}
#[allow(dead_code, unused_mut)]
impl Default for Config {
    fn default() -> Self {
        // let (tx, mut rx) = tokio::sync::mpsc::channel::<Animation>(32);

        Config {
            database: DatabaseConfig::default(),
            web: WebConfig::default(),
            module_enable: LoadModuleConfig::default(),
            command_comms: CompactSender::new(),
            current_data: CurrentAnimationData::default(),
            day_night: Arc::new(Mutex::new(DayNightConfig::default())),
        }
    }
}
#[allow(dead_code, unused_mut)]
impl From<TOMLConfig> for Config {
    fn from(a: TOMLConfig) -> Self {
        Config {
            database: a.database,
            web: a.web,
            module_enable: a.modules,
            command_comms: CompactSender::new(),
            current_data: CurrentAnimationData::default(),
            day_night: Arc::new(Mutex::new(a.day_night)),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            file_path: "/home/pi/Lights/db/sqlite.db".to_string(),
        }
    }
}

#[allow(dead_code, unused_mut)]
pub fn read_or_create_config<P: AsRef<Path>>(path: P) -> io::Result<Config> {
    let mut toml_config = TOMLConfig::default();
    if path.as_ref().exists() {
        let content = fs::read_to_string(&path)?;
        toml_config = toml::from_str(&content).unwrap_or_default();
    } else {
        let toml_string = toml::to_string(&toml_config).unwrap();
        let mut file = fs::File::create(&path)?;
        file.write_all(toml_string.as_bytes())?;
    }
    let mut config: Config = toml_config.into();
    Ok(config)
}
