use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use axum::{
    extract,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Router,
};

#[allow(dead_code, unused_imports)]
use tokio::sync::mpsc::{channel, Receiver, Sender};
use toml;

use crate::command::ChangeLighting;
use crate::database::initialize::AppState;
use crate::thread_utils::{CompactSender, Notifier};

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

#[derive(Debug, Clone)]
pub struct CurrentAnimationData {
    pub brightness: Notifier<u8>,
    pub animation_index: Notifier<i32>,
    pub frame_index: Notifier<usize>,
    pub animation_speed: Notifier<f64>,
}

impl Default for CurrentAnimationData {
    fn default() -> Self {
        CurrentAnimationData {
            brightness: Notifier::new(100),
            animation_index: Notifier::new(0),
            frame_index: Notifier::new(0),
            animation_speed: Notifier::new(24.0),
        }
    }
}

impl CurrentAnimationData {
    pub fn to_json(&self) -> serde_json::Value {
        let brightness = *self.brightness.receving_channel.borrow();
        let animation_index = *self.animation_index.receving_channel.borrow();
        let frame_index = *self.frame_index.receving_channel.borrow();
        let animation_speed = *self.animation_speed.receving_channel.borrow();

        return json!({
            "brightness": brightness,
            "animation_index": animation_index,
            "frame_index": frame_index,
            "animation_speed": animation_speed,
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
    pub enabled: bool,
}

impl Default for DayNightConfig {
    fn default() -> Self {
        DayNightConfig {
            day_hour: 6,
            day_brightness: 1,
            night_hour: 15,
            night_brightness: 100,
            enabled: true,
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

fn convert_to<T>(value: &str) -> Result<T, Response>
where
    T: FromStr,
    <T as FromStr>::Err: ToString,
{
    match value.parse::<T>() {
        Ok(value) => return Ok(value),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                json!({"value":value, "error":err.to_string()}).to_string(),
            )
                .into_response());
        }
    };
}

pub async fn change_setting(
    extract::Path((setting, value)): extract::Path<(String, String)>,
    extract::State(state): extract::State<Arc<AppState>>,
) -> Response {
    let mut day_night = state.time_of_day_config.lock().unwrap();
    let temp_number: u8 = match convert_to(&value) {
        Err(err) => return err,
        Ok(value) => value,
    };

    let old_value = match setting.to_ascii_lowercase().as_str(){
        "day_hour" => day_night.day_hour.to_string(),
        "night_hour" => day_night.night_hour.to_string(),
        "day_brightness" => day_night.day_brightness.to_string(),
        "night_brightness" => day_night.night_brightness.to_string(),
        "enabled" => day_night.enabled.to_string(),
        _ => String::from("Error, not a valid setting. Valid options are [day_hour,day_brightness,night_hour,night_brightness]"),
    };

    match setting.to_ascii_lowercase().as_str() {
        "day_hour" => {
            day_night.day_hour = temp_number.clamp(0, 24) as u32;
        }
        "night_hour" => {
            day_night.night_hour = temp_number.clamp(0, 24) as u32;
        }
        "day_brightness" => {
            day_night.day_brightness = temp_number.clamp(0, 255);
        }
        "night_brightness" => {
            day_night.night_brightness = temp_number.clamp(0, 255);
        }
        "enabled" => {
            day_night.enabled = temp_number.clamp(0, 1) != 0;
        }
        _ => {}
    };

    return (
        StatusCode::OK,
        json!({"setting": setting, "value":value, "old value":old_value}).to_string(),
    )
        .into_response();
}

pub async fn get_setting(
    extract::Path(setting): extract::Path<String>,
    extract::State(state): extract::State<Arc<AppState>>,
) -> Response {
    // let mut old_value = String::new();
    let day_night = state.time_of_day_config.lock().unwrap();

    let old_value = match setting.to_ascii_lowercase().as_str(){
        "day_hour" => day_night.day_hour.clone().to_string(),
        "night_hour" => day_night.night_hour.clone().to_string(),
        "day_brightness" => day_night.day_brightness.clone().to_string(),
        "night_brightness" => day_night.night_brightness.clone().to_string(),
        "enabled" => day_night.enabled.clone().to_string(),
        _ => String::from("Error, not a valid setting. Valid options are [day_hour,day_brightness,night_hour,night_brightness]"),
    };

    return (
        StatusCode::OK,
        json!({"setting": setting, "value":old_value}).to_string(),
    )
        .into_response();
}

pub fn router(index: &mut HashMap<&'static str, &str>, state: Arc<AppState>) -> Router {
    let app = Router::new()
        .route("/:setting/:value", post(change_setting))
        .route("/:setting", get(get_setting))
        // .route("/", get(get_animations))
        // .route("/:id", get(get_animation_id))
        // .route("/:id", delete(delete_animation_id))
        // .route("/:id", post(set_animation))
        // .route("/brightness/:id", post(set_brightness))
        // .route("/speed/:fps", post(set_fps))
        .with_state(state);

    index.insert("/settings/:setting/:value", "POST");
    index.insert("/settings/:setting", "GET");

    return app;
}
