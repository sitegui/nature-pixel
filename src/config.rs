use anyhow::Result;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub port: u16,
    pub map_size: usize,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config: Config = serde_json::from_str(&fs::read_to_string("config.json")?)?;
        Ok(config)
    }
}
