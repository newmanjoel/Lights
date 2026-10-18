use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[allow(dead_code, unused_imports)]
use tokio::sync::mpsc::{channel, Receiver, Sender};
use toml;

use crate::command::ChangeLighting;
use crate::database::initialize::AppState;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TOMLConfig {
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub web: WebConfig,
    #[serde(default)]
    pub modules: LoadModuleConfig,
    #[serde(default)]
    pub day_night: DayNightConfig,
}

#[derive(Debug)]
pub struct CompactSender<T> {
    pub sending_channel: tokio::sync::mpsc::Sender<T>,
    pub receving_channel: tokio::sync::mpsc::Receiver<T>,
}
impl<T> CompactSender<T> {
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel::<T>(32);
        CompactSender {
            sending_channel: tx,
            receving_channel: rx,
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub database: Arc<Mutex<DatabaseConfig>>,
    pub web: Arc<Mutex<WebConfig>>,
    pub module_enable: Arc<Mutex<LoadModuleConfig>>,
    pub command_comms: CompactSender<ChangeLighting>,
    pub current_data: CurrentAnimationData,
    pub day_night: Arc<Mutex<DayNightConfig>>,
    pub file_path: Arc<Mutex<String>>,
}

#[allow(dead_code)]
impl Config {
    pub fn to_toml(&self) -> TOMLConfig {
        let database = self.database.lock().unwrap().clone();
        let web = self.web.lock().unwrap().clone();
        let modules = self.module_enable.lock().unwrap().clone();
        let day_night = self.day_night.lock().unwrap().clone();
        TOMLConfig {
            database,
            web,
            modules,
            day_night,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        let database = self.database.lock().unwrap().clone();
        let web = self.web.lock().unwrap().clone();
        let modules = self.module_enable.lock().unwrap().clone();
        let day_night = self.day_night.lock().unwrap().clone();
        json!({
            "database": database,
            "web": web,
            "modules": modules,
            "day_night": day_night,
        })
    }

}

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
            database: Arc::new(Mutex::new(DatabaseConfig::default())),
            web: Arc::new(Mutex::new(WebConfig::default())),
            module_enable: Arc::new(Mutex::new(LoadModuleConfig::default())),
            command_comms: CompactSender::new(),
            current_data: CurrentAnimationData::default(),
            day_night: Arc::new(Mutex::new(DayNightConfig::default())),
            file_path: Arc::new(Mutex::new("./config.toml".to_string())),
        }
    }
}
#[allow(dead_code, unused_mut)]
impl From<TOMLConfig> for Config {
    fn from(a: TOMLConfig) -> Self {
        Config {
            database: Arc::new(Mutex::new(a.database)),
            web: Arc::new(Mutex::new(a.web)),
            module_enable: Arc::new(Mutex::new(a.modules)),
            command_comms: CompactSender::new(),
            current_data: CurrentAnimationData::default(),
            day_night: Arc::new(Mutex::new(a.day_night)),
            file_path: Arc::new(Mutex::new("./config.toml".to_string())),
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

#[allow(unused_mut)]
pub fn read_or_create_config(path_str: &str) -> io::Result<Config> {
    let path: &Path = Path::new(path_str);
    let mut toml_config = TOMLConfig::default();
    if path.exists() {
        let content = fs::read_to_string(&path)?;
        toml_config = toml::from_str(&content).unwrap_or_default();
    } else {
        let toml_string = toml::to_string(&toml_config).unwrap();
        let mut file = fs::File::create(&path)?;
        file.write_all(toml_string.as_bytes())?;
    }
    let mut config: Config = toml_config.into();
    *config.file_path.lock().unwrap() = path_str.to_string();
    Ok(config)
}

pub fn router(index: &mut HashMap<&'static str, &str>, state: Arc<AppState>) -> axum::Router {

    let app = axum::Router::new()
        .route("/", get(get_all_settings).put(update_full_config).post(update_full_config))
        .route("/database", get(get_database_config).put(update_database).post(update_database))
        .route("/web", get(get_web_config).put(update_web).post(update_web))
        .route("/modules", get(get_modules_config).put(update_modules).post(update_modules))
        .route("/day_night", get(get_day_night_config).put(update_day_night).post(update_day_night))
        .with_state(state);

    index.insert("/setting", "GET,PUT,POST");
    index.insert("/setting/database", "GET,PUT,POST");
    index.insert("/setting/web", "GET,PUT,POST");
    index.insert("/setting/modules", "GET,PUT,POST");
    index.insert("/setting/day_night", "GET,PUT,POST");
    return app;
}

const CONFIG_EXAMPLE: &str = r#"
{
    "database": {"file_path": "/home/pi/Lights/db/sqlite.db"},
    "web": {"port": 3000, "interface": "0.0.0.0"},
    "modules": {"on_raspberry_pi": false, "webserver": true, "lights": true, "timed_brightness": true},
    "day_night": {"day_hour": 6, "day_brightness": 1, "night_hour": 16, "night_brightness": 100}
}
"#;

type ConfigError = (StatusCode, String);

fn parse_error(example: &str, extra: Value) -> ConfigError {
    let mut error = json!({"error": "parsing json", "example": example});
    if let Some(serde_map) = error.as_object_mut() {
        if let Some(extra_map) = extra.as_object() {
            for (key, value) in extra_map {
                serde_map.insert(key.clone(), value.clone());
            }
        }
    }
    (StatusCode::BAD_REQUEST, error.to_string())
}

fn extract_toml_config(payload: String) -> Result<TOMLConfig, ConfigError> {
    let json_result = serde_json::from_str::<Value>(&payload).map_err(|error| {
        parse_error(CONFIG_EXAMPLE, json!({"payload": payload, "debug": error.to_string()}))
    })?;
    match TOMLConfig::deserialize(json_result) {
        Ok(config) => Ok(config),
        Err(_) => Err(parse_error(
            CONFIG_EXAMPLE,
            json!({"debug": "invalid json"}),
        )),
    }
}

#[allow(dead_code)]
fn merge_section<T: Default + for<'a> Deserialize<'a> + Clone>(current: &T, update: &Value) -> T {
    #[allow(dead_code)]
    match serde_json::from_value::<T>(update.clone()).ok() {
        Some(full) => full,
        None => current.clone(),
    }
}

#[allow(dead_code)]
async fn get_database_config(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::OK, json!({ "database": state.database.lock().unwrap().clone() }).to_string())
}

#[allow(dead_code)]
async fn update_database(State(state): State<Arc<AppState>>, payload: String) -> impl IntoResponse {
    match extract_toml_config(payload) {
        Ok(incoming) => {
            let new_database = incoming.database;
            *state.database.lock().unwrap() = new_database.clone();
            (StatusCode::OK, json!({"database": state.database.lock().unwrap().clone()}).to_string())
        }
        Err(error) => error,
    }
}

#[allow(dead_code)]
async fn get_web_config(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::OK, json!({ "web": state.web.lock().unwrap().clone() }).to_string())
}

#[allow(dead_code)]
async fn update_web(State(state): State<Arc<AppState>>, payload: String) -> impl IntoResponse {
    match extract_toml_config(payload) {
        Ok(incoming) => {
            let new_web = incoming.web;
            let mut web_m = state.web.lock().unwrap();
            *web_m = new_web.clone();
            match web_m.save_to_file() {
                Ok(_) => (StatusCode::OK, json!({"web": *web_m}).to_string()),
                Err(error) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": format!("{error:?}")}).to_string(),
                ),
            }
        }
        Err(error) => error,
    }
}

#[allow(dead_code)]
async fn get_modules_config(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::OK, json!({ "modules": state.module_enable.lock().unwrap().clone() }).to_string())
}

#[allow(dead_code)]
async fn update_modules(State(state): State<Arc<AppState>>, payload: String) -> impl IntoResponse {
    match extract_toml_config(payload) {
        Ok(incoming) => {
            let new_modules = incoming.modules;
            let mut modules_m = state.module_enable.lock().unwrap();
            *modules_m = new_modules.clone();
            match modules_m.save_to_file() {
                Ok(_) => (StatusCode::OK, json!({"modules": *modules_m}).to_string()),
                Err(error) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": format!("{error:?}")}).to_string(),
                ),
            }
        }
        Err(error) => error,
    }
}

#[allow(dead_code)]
async fn get_day_night_config(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::OK, json!({ "day_night": state.day_night.lock().unwrap().clone() }).to_string())
}

#[allow(dead_code)]
async fn update_day_night(State(state): State<Arc<AppState>>, payload: String) -> impl IntoResponse {
    match extract_toml_config(payload) {
        Ok(incoming) => {
            let new_day_night = incoming.day_night;
            let mut day_night_m = state.day_night.lock().unwrap();
            *day_night_m = new_day_night.clone();
            match day_night_m.save_to_file() {
                Ok(_) => (StatusCode::OK, json!({"day_night": *day_night_m}).to_string()),
                Err(error) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": format!("{error:?}")}).to_string(),
                ),
            }
        }
        Err(error) => error,
    }
}

async fn get_all_settings(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::OK, state.to_json().to_string())
}

async fn update_full_config(State(state): State<Arc<AppState>>, payload: String) -> impl IntoResponse {
    match extract_toml_config(payload) {
        Ok(incoming) => {
            let new_database = incoming.database;
            let new_web = incoming.web;
            let new_modules = incoming.modules;
            let new_day_night = incoming.day_night;

            let mut database_m = state.database.lock().unwrap();
            let mut web_m = state.web.lock().unwrap();
            let mut modules_m = state.module_enable.lock().unwrap();
            let mut day_night_m = state.day_night.lock().unwrap();

            *database_m = new_database.clone();
            *web_m = new_web.clone();
            *modules_m = new_modules.clone();
            *day_night_m = new_day_night.clone();

            match modules_m.save_to_file() {
                Ok(_) => (StatusCode::OK, state.to_json().to_string()),
                Err(error) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": format!("{error:?}")}).to_string(),
                ),
            }
        }
        Err(error) => error,
    }
}

impl AppState {
    #[allow(dead_code)]
    pub fn to_json(&self) -> serde_json::Value {
        let database = self.database.lock().unwrap().clone();
        let web = self.web.lock().unwrap().clone();
        let modules = self.module_enable.lock().unwrap().clone();
        let day_night = self.day_night.lock().unwrap().clone();
        json!({
            "database": database,
            "web": web,
            "modules": modules,
            "day_night": day_night,
        })
    }
}

#[allow(dead_code)]
impl DatabaseConfig {
    pub fn save_to_file(&self) -> io::Result<()> {
        let _toml_string = toml::to_string(self).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Could not serialize config: {e}"))
        })?;
        Ok(())
    }
}

#[allow(dead_code)]
impl WebConfig {
    pub fn save_to_file(&self) -> io::Result<()> {
        let _toml_string = toml::to_string(self).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Could not serialize config: {e}"))
        })?;
        Ok(())
    }
}

#[allow(dead_code)]
impl LoadModuleConfig {
    pub fn save_to_file(&self) -> io::Result<()> {
        let _toml_string = toml::to_string(self).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Could not serialize config: {e}"))
        })?;
        Ok(())
    }
}

impl DayNightConfig {
    #[allow(dead_code)]
    pub fn save_to_file(&self) -> io::Result<()> {
        let _toml_string = toml::to_string(self).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Could not serialize config: {e}"))
        })?;
        Ok(())
    }
}


