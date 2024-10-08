use std::fs::File;

use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub db_url: String,
    pub base_url: String,
    pub auth: AuthConfig,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub sk: String,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        // read from "app.yml" or "/etc/config/app.yml" or env::var("CHAT_CONFIG")
        match (
            File::open("chat.yml"),
            File::open("/etc/config/chat.yml"),
            std::env::var("CHAT_CONFIG"),
        ) {
            (Ok(file), _, _) => Ok(serde_yaml::from_reader(file)?),
            (_, Ok(file), _) => Ok(serde_yaml::from_reader(file)?),
            (_, _, Ok(path)) => Ok(serde_yaml::from_str(&path)?),
            _ => bail!("no config file found"),
        }
    }
}
