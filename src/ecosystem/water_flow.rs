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
        let water_flows =
            Self::determine_water_flows(config.water_flow_max_radius, &map.read().unwrap());

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

    /// Determine to which neighbors each cell will flow
    fn determine_water_flows(max_radius: usize, map: &Map) -> Vec<WaterFlow> {
        let mut flows = Vec::new();

        for radius in (1..=max_radius).rev() {
            Self::append_water_flows(radius, map, &mut flows);
        }

        tracing::info!("Prepared {} water flows", flows.len());
        flows
    }

    /// Determine to which neighbor at a given radius each cell will flow
    fn append_water_flows(radius: usize, map: &Map, flows: &mut Vec<WaterFlow>) {
        let cells = map.cells();
        let mut flows = Vec::new();

        for ((i, j), cell) in cells.indexed_iter() {
            let mut lowest_neighbor_height = cell.height();
            let mut lowest_neighbor_coordinates = (i, j);

            let start_i = i.saturating_sub(radius);
            let end_i = (i + radius + 1).min(map.size());
            for neighbor_i in start_i..end_i {
                let height = cells[(neighbor_i, j)].height();
                if height < lowest_neighbor_height {
                    lowest_neighbor_height = height;
                    lowest_neighbor_coordinates = (neighbor_i, j);
                }
            }

            if lowest_neighbor_coordinates != (i, j) {
                flows.push(WaterFlow {
                    from: (i, j),
                    to: lowest_neighbor_coordinates,
                });
            }
        }

        tracing::info!("Prepared {} water flows", flows.len());
        flows
    }
}
