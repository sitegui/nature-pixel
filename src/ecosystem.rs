mod insect;
mod water;

use crate::config::Config;
use crate::map::Map;
use std::sync::{Arc, RwLock};

/// Continuously update the map, simulating all the living things
pub async fn spawn_ecosystem(config: Arc<Config>, map: Arc<RwLock<Map>>) {
    loop {
        // TODO
    }
}
