use anyhow::Result;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub amphibian_destination_radius: usize,
    pub amphibian_eating_radius: usize,
    pub amphibian_mating_radius: usize,
    pub amphibian_starvation_delay_seconds: u64,
    pub amphibian_tick_seconds: u64,
    pub height_map: String,
    pub insect_destination_radius: usize,
    pub insect_eating_radius: usize,
    pub insect_mating_radius: usize,
    pub insect_starvation_delay_seconds: u64,
    pub insect_tick_seconds: u64,
    pub long_pooling_seconds: u64,
    pub map_size: usize,
    pub port: u16,
    pub snake_a_max_size: usize,
    pub snake_a_move_ratio: f64,
    pub snake_b_max_size: usize,
    pub snake_b_move_ratio: f64,
    pub snake_c_max_size: usize,
    pub snake_c_move_ratio: f64,
    pub snake_eating_radius: usize,
    pub snake_min_size: usize,
    pub snake_starvation_delay_seconds: u64,
    pub snake_tick_seconds: u64,
    pub water_evaporation_ratio: f64,
    pub water_evaporation_tick_seconds: u64,
    pub water_flow_max_radius: usize,
    pub water_flow_tick_seconds: u64,
    pub water_in_atmosphere_ratio: f64,
    pub water_max_cycle_seconds: u64,
    pub water_max_rain_radius: usize,
    pub water_min_cycle_seconds: u64,
    pub water_rain_ratio: f64,
    pub water_rain_tick_seconds: u64,
    pub water_thickness: u8,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config: Config = serde_json::from_str(&fs::read_to_string("config/config.json")?)?;
        Ok(config)
    }
}
