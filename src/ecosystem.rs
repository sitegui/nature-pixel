mod insect;
mod water_cycle;
mod water_flow;

use crate::config::Config;
use crate::ecosystem::water_cycle::WaterCycleSystem;
use crate::ecosystem::water_flow::WaterFlowSystem;
use crate::map::Map;
use std::sync::{Arc, RwLock};

/// Continuously update the map, simulating all the living things
pub fn spawn_ecosystem(config: Arc<Config>, map: Arc<RwLock<Map>>) {
    tokio::spawn(WaterCycleSystem::new(&config, map.clone()).run());
    tokio::spawn(WaterFlowSystem::new(&config, map.clone()).run());

    // tokio::spawn(async move {
    //     let mut map = map.write().unwrap();
    //     map.cells_mut()
    //         .indexed_iter_mut()
    //         .for_each(|(coords, cell)| {
    //             if (coords.0 + coords.1) % 2 == 0 {
    //                 cell.set_water(CellWater::Shallow);
    //             }
    //         });
    //     map.notify_update();
    // });

    // tokio::spawn(async move {
    //     loop {
    //         {
    //             let mut map = map.write().unwrap();
    //             let size = map.size();
    //             map.cells_mut()[(size / 4, size / 4)].set_water(CellWater::Shallow);
    //             map.cells_mut()[(3 * size / 4, size / 4)].set_water(CellWater::Shallow);
    //             map.cells_mut()[(size / 4, 3 * size / 4)].set_water(CellWater::Shallow);
    //             map.cells_mut()[(3 * size / 4, 3 * size / 4)].set_water(CellWater::Shallow);
    //             map.notify_update();
    //         }
    //
    //         time::sleep(Duration::from_millis(100)).await;
    //     }
    // });
}
