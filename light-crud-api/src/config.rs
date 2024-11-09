use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use toml;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub database: DatabaseConfig,
    pub web: WebConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseConfig {
    pub file_path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebConfig {
    pub port: i32,
}

impl Default for WebConfig {
    fn default() -> Self {
        WebConfig { port: 3000 }
    }
}
impl Default for Config {
    fn default() -> Self {
        Config {
            database: DatabaseConfig::default(),
            web: WebConfig::default(),
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
    if path.as_ref().exists() {
        let content = fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content).unwrap_or_default();
        Ok(config)
    } else {
        let config = Config::default();
        let toml_string = toml::to_string(&config).unwrap();
        let mut file = fs::File::create(&path)?;
        file.write_all(toml_string.as_bytes())?;
        Ok(config)
    }
}
