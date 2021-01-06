
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};

use crate::utils;

#[derive(Serialize, Deserialize, Debug)]
pub struct Server {
    pub name: String,
    pub server: String,
    pub password: String,
    pub method: String,
    pub timeout: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub select: usize,
    pub autostart: bool,
    pub startminimized: bool,
    pub local_addr: String,
    pub servers: Vec<Server>,
}

impl Config {
    pub fn load() -> Result<Config> {
        let filename = find_config();
        let str = fs::read_to_string(filename).context("unable to open config file to read")?;
        let config: Config = serde_json::from_str(&str).context("unable to decode config file")?;
        Ok(config)
    }

    pub fn save(cfg: &Config) -> Result<()> {
        let filename = find_config();
        let str = serde_json::to_string_pretty(&cfg).context("unable to open config file to write")?;
        fs::write(filename, str).context("unable to write config file")?;
        Ok(())
    }
}

// try the config file in current dir first, it is easier for debugging
fn find_config() -> PathBuf {
    if Path::new("russ.json").exists() {
        PathBuf::from("russ.json")
    } else {
        let mut file = utils::exe_path();
        file.push("russ.json");
        file
    }
}
