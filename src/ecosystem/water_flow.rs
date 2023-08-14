use crate::config::Config;
use crate::map::Map;
use ndarray::Array2;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time;

#[derive(Debug)]
pub struct WaterFlowSystem {
    map: Arc<RwLock<Map>>,
    water_flows: Array2<WaterFlow>,
    tick_sleep: Duration,
    tick: usize,
}

#[derive(Debug)]
struct WaterFlow {
    /// The cell coordinates that want to receive water from this one, in priority order
    targets: Box<[(usize, usize)]>,
    /// The tick in which this cell last received a water flowing into it. This is used so that
    /// each cell can only give water if it hasn't received water this tick
    last_received_tick: std::cell::Cell<usize>,
}

impl WaterFlowSystem {
    pub fn new(config: &Config, map: Arc<RwLock<Map>>) -> Self {
        let water_flows =
            Self::determine_water_flows(config.water_flow_max_radius, &map.read().unwrap());

        Self {
            map,
            water_flows,
            tick_sleep: Duration::from_secs(config.water_flow_tick_seconds),
            tick: 0,
        }
    }

    pub async fn run(mut self) {
        loop {
            self.flow();
            time::sleep(self.tick_sleep).await;
        }
    }

    fn flow(&mut self) {
        self.tick += 1;

        let this_tick = self.tick;
        let mut map = self.map.write().unwrap();
        let cells = map.cells_mut();
        let mut flowed = 0;

        for (source, flow) in self.water_flows.indexed_iter() {
            if flow.last_received_tick.get() == this_tick {
                // Cannot give water if already received at this time
                continue;
            }

            if let Some(drier) = cells[source].water().drier() {
                for &target in flow.targets.iter() {
                    let target_cell = &mut cells[target];
                    if let Some(wetter) = target_cell.water().wetter() {
                        target_cell.set_water(wetter);
                        cells[source].set_water(drier);
                        flowed += 1;
                        self.water_flows[target].last_received_tick.set(this_tick);
                        break;
                    }
                }
            }
        }

        if flowed > 0 {
            tracing::info!("Flowed {} water", flowed);
            map.notify_update();
        }
    }

    /// Determine to which neighbors each cell will flow
    fn determine_water_flows(max_radius: usize, map: &Map) -> Array2<WaterFlow> {
        let cells = map.cells();
        let size = map.size();

        Array2::from_shape_fn(map.cells().dim(), |(i, j)| {
            let height = cells[(i, j)].height();
            let mut targets = Vec::new();

            let mut maybe_add_target = |target, radius| {
                let target_height = cells[target].height();
                if target_height < height {
                    targets.push((radius, target_height, target));
                }
            };

            let start_i = i.saturating_sub(max_radius);
            let end_i = (i + max_radius + 1).min(size);
            for target_i in start_i..end_i {
                let delta_i = target_i.abs_diff(i);
                let max_delta_j = max_radius - delta_i;
                let start_j = j.saturating_sub(max_delta_j);
                let end_j = (j + max_delta_j + 1).min(size);

                for target_j in start_j..end_j {
                    let delta_j = target_j.abs_diff(j);
                    maybe_add_target((target_i, target_j), delta_i + delta_j);
                }
            }

            // Consider first the targets closer to the cell (lesser radius). In case of a tie,
            // consider first the targets with the least height
            targets.sort_by_key(|&(radius, height, _)| (radius, height));

            WaterFlow {
                targets: targets.into_iter().map(|(_, _, target)| target).collect(),
                last_received_tick: std::cell::Cell::new(0),
            }
        })
    }
}
