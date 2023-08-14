use crate::config::Config;
use crate::map::Map;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time;

#[derive(Debug)]
pub struct WaterFlowSystem {
    map: Arc<RwLock<Map>>,
    water_flows: Vec<WaterFlow>,
    tick: Duration,
}

#[derive(Debug)]
struct WaterFlow {
    from: (usize, usize),
    to: (usize, usize),
}

impl WaterFlowSystem {
    pub fn new(config: &Config, map: Arc<RwLock<Map>>) -> Self {
        let water_flows = Self::determine_water_flows(&map.read().unwrap());

        Self {
            map,
            water_flows,
            tick: Duration::from_secs(config.water_flow_tick_seconds),
        }
    }

    pub async fn run(mut self) {
        loop {
            self.flow();
            time::sleep(self.tick).await;
        }
    }

    fn flow(&mut self) {
        let mut map = self.map.write().unwrap();
        let cells = map.cells_mut();
        let mut flowed = 0;

        for water_flow in &self.water_flows {
            let from_water = &cells[water_flow.from].water();
            let to_water = &cells[water_flow.to].water();

            if let Some((drier, wetter)) = from_water.drier().zip(to_water.wetter()) {
                cells[water_flow.from].set_water(drier);
                cells[water_flow.to].set_water(wetter);
                flowed += 1;
            }
        }

        if flowed > 0 {
            tracing::info!("Flowed {} water", flowed);
            map.notify_update();
        }
    }

    /// Determine to which neighbor each cell will flow
    fn determine_water_flows(map: &Map) -> Vec<WaterFlow> {
        let cells = map.cells();
        let mut flows = Vec::new();

        for ((i, j), cell) in cells.indexed_iter() {
            let mut lowest_neighbor_height = cell.height();
            let mut lowest_neighbor_coordinates = (i, j);

            let mut update_neighbor = |coordinates| {
                if let Some(cell) = cells.get(coordinates) {
                    if cell.height() < lowest_neighbor_height {
                        lowest_neighbor_height = cell.height();
                        lowest_neighbor_coordinates = coordinates;
                    }
                }
            };

            update_neighbor((i + 1, j));
            update_neighbor((i, j + 1));
            update_neighbor((i.saturating_sub(1), j));
            update_neighbor((i, j.saturating_sub(1)));

            if lowest_neighbor_coordinates != (i, j) {
                flows.push(WaterFlow {
                    from: (i, j),
                    to: lowest_neighbor_coordinates,
                });
            }
        }

        flows
    }
}
