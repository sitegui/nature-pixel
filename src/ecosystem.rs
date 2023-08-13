mod insect;
mod water;

use crate::config::Config;
use crate::ecosystem::water::WaterSystem;
use crate::map::Map;
use std::sync::{Arc, RwLock};

/// Continuously update the map, simulating all the living things
pub fn spawn_ecosystem(config: Arc<Config>, map: Arc<RwLock<Map>>) {
    tokio::spawn(WaterSystem::new(config, map).run());
}
