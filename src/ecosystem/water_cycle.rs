use crate::config::Config;
use crate::map::Map;
use crate::point::Point;
use itertools::Itertools;
use rand::distributions::Bernoulli;
use rand::prelude::{Distribution, SliceRandom, SmallRng};
use rand::{Rng, SeedableRng};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time;

#[derive(Debug)]
pub struct WaterCycleSystem {
    min_cycle: Duration,
    max_cycle: Duration,
    evaporation_ratio: f64,
    evaporation_tick: Duration,
    rain_ratio: f64,
    rain_tick: Duration,
    max_rain_radius: usize,
    map: Arc<RwLock<Map>>,
    rng: SmallRng,
    atmosphere_water: i32,
}

impl WaterCycleSystem {
    pub fn new(config: &Config, map: Arc<RwLock<Map>>) -> Self {
        let size = map.read().unwrap().size() as f64;
        let atmosphere_water = config.water_in_atmosphere_ratio * size * size;
        Self {
            min_cycle: Duration::from_secs(config.water_min_cycle_seconds),
            max_cycle: Duration::from_secs(config.water_max_cycle_seconds),
            evaporation_ratio: config.water_evaporation_ratio,
            evaporation_tick: Duration::from_secs(config.water_evaporation_tick_seconds),
            rain_ratio: config.water_rain_ratio,
            rain_tick: Duration::from_secs(config.water_rain_tick_seconds),
            max_rain_radius: config.water_max_rain_radius,
            map,
            rng: SmallRng::from_entropy(),
            atmosphere_water: atmosphere_water.round() as i32,
        }
    }

    pub async fn run(mut self) {
        loop {
            self.rain().await;
            self.evaporate().await;
        }
    }

    async fn evaporate(&mut self) {
        let cycle_duration = self.rng.gen_range(self.min_cycle..=self.max_cycle);
        let num_ticks =
            (cycle_duration.as_secs_f64() / self.evaporation_tick.as_secs_f64()).ceil() as i32;
        let ratio_per_tick = 1.0 - (1.0 - self.evaporation_ratio).powf(1.0 / num_ticks as f64);
        tracing::info!(
            "Will evaporate {}% of the water each tick",
            100.0 * ratio_per_tick
        );
        let Ok(random) = Bernoulli::new(ratio_per_tick) else { return };

        for _ in 0..num_ticks {
            {
                let mut map = self.map.write().unwrap();

                for cell in map.cells_mut() {
                    if let Some(drier) = cell.water().drier() {
                        if random.sample(&mut self.rng) {
                            cell.set_water(drier);
                            self.atmosphere_water += 1;
                        }
                    }
                }

                map.notify_update();
            }

            time::sleep(self.evaporation_tick).await;
        }
    }

    async fn rain(&mut self) {
        let cycle_duration = self.rng.gen_range(self.min_cycle..=self.max_cycle);
        let num_ticks = (cycle_duration.as_secs_f64() / self.rain_tick.as_secs_f64()).ceil() as i32;
        let ratio_per_tick = 1.0 - (1.0 - self.rain_ratio).powf(1.0 / num_ticks as f64);
        tracing::info!(
            "Will rain {}% of the water each tick",
            100.0 * ratio_per_tick
        );

        for _ in 0..num_ticks {
            {
                let mut map = self.map.write().unwrap();

                let mut remaining_rain =
                    (self.atmosphere_water as f64 * ratio_per_tick).ceil() as i32;
                let mut radius = 0;
                let center_x = self.rng.gen_range(0..map.size());
                let center_y = self.rng.gen_range(0..map.size());
                let center = Point::new(center_x, center_y);
                while remaining_rain > 0 && radius <= self.max_rain_radius {
                    let mut candidates = center.circumference(radius, map.size()).collect_vec();
                    candidates.shuffle(&mut self.rng);
                    Self::add_rain(
                        &mut map,
                        &candidates,
                        &mut remaining_rain,
                        &mut self.atmosphere_water,
                    );

                    radius += 1;
                }

                map.notify_update();
            }

            time::sleep(self.rain_tick).await;
        }
    }

    fn add_rain(
        map: &mut Map,
        candidates: &[Point],
        remaining_rain: &mut i32,
        atmosphere_water: &mut i32,
    ) {
        for &candidate in candidates {
            if *remaining_rain <= 0 {
                break;
            }

            let cell = &mut map.cells_mut()[candidate];
            if let Some(wetter) = cell.water().wetter() {
                cell.set_water(wetter);
                *remaining_rain -= 1;
                *atmosphere_water -= 1;
            }
        }
    }
}
