use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use toml;

use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::database::frame::Frame;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TOMLConfig {
    pub database: DatabaseConfig,
    pub web: WebConfig,
    pub debug: DebugConfig,
}

#[derive(Debug)]
pub struct Config {
    pub database: DatabaseConfig,
    pub web: WebConfig,
    pub debug: DebugConfig,
    pub sending_channel: tokio::sync::mpsc::Sender<crate::database::frame::Frame>,
    pub receving_channel: tokio::sync::mpsc::Receiver<crate::database::frame::Frame>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatabaseConfig {
    pub file_path: String,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct DebugConfig {
    pub on_raspberry_pi: bool,
    pub enable_webserver: bool,
    pub enable_lights: bool,
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
impl Default for DebugConfig {
    fn default() -> Self {
        DebugConfig {
            on_raspberry_pi: false,
            enable_webserver: false,
            enable_lights: false,
        }
    }
}
impl Default for Config {
    fn default() -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Frame>(32);
        Config {
            database: DatabaseConfig::default(),
            web: WebConfig::default(),
            debug: DebugConfig::default(),
            sending_channel: tx,
            receving_channel: rx,
        }
    }
}
impl From<TOMLConfig> for Config {
    fn from(a: TOMLConfig) -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Frame>(32);
        Config {
            database: a.database,
            web: a.web,
            debug: a.debug,
            sending_channel: tx,
            receving_channel: rx,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            file_path: "/home/joel/GH/Lights/db/sqlite.db".to_string(),
        }
    }
}

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
