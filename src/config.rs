use anyhow::Result;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub port: u16,
    pub map_size: usize,
    pub long_pooling_seconds: u64,
    pub water_min_cycle_seconds: u64,
    pub water_max_cycle_seconds: u64,
    pub water_evaporation_ratio: f64,
    pub water_evaporation_tick_seconds: u64,
    pub water_rain_ratio: f64,
    pub water_rain_tick_seconds: u64,
    pub water_max_rain_radius: usize,
    pub water_in_atmosphere: i32,
    pub water_height_map: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config: Config = serde_json::from_str(&fs::read_to_string("config/config.json")?)?;
        Ok(config)
    }
}
