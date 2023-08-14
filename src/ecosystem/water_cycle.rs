use crate::config::Config;
use crate::map::Map;
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
            atmosphere_water: config.water_in_atmosphere,
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
                while remaining_rain > 0 && radius <= self.max_rain_radius {
                    let mut candidates = Self::circle(center_x, center_y, radius, map.size());
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
        candidates: &[[usize; 2]],
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

    /// Determine all cells with exactly `radius` taxi-distance of the center
    fn circle(center_x: usize, center_y: usize, radius: usize, size: usize) -> Vec<[usize; 2]> {
        if radius == 0 {
            return vec![[center_x, center_y]];
        }

        let mut points = Vec::with_capacity(4 * radius);
        let mut maybe_push = |x: isize, y: isize| {
            if x >= 0 && x < size as isize && y >= 0 && y < size as isize {
                points.push([x as usize, y as usize]);
            }
        };

        let center_x = center_x as isize;
        let center_y = center_y as isize;
        let radius = radius as isize;
        maybe_push(center_x - radius, center_y);
        maybe_push(center_x + radius, center_y);
        for delta_x in (-radius + 1)..radius {
            let delta_y = radius - delta_x.abs();
            maybe_push(center_x + delta_x, center_y + delta_y);
            maybe_push(center_x + delta_x, center_y - delta_y);
        }

        points
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle() {
        assert_eq!(WaterCycleSystem::circle(10, 20, 0, 100), vec![[10, 20]]);
        assert_eq!(
            WaterCycleSystem::circle(10, 20, 1, 100),
            vec![[9, 20], [11, 20], [10, 21], [10, 19]]
        );
        assert_eq!(
            WaterCycleSystem::circle(10, 20, 2, 100),
            vec![
                [8, 20],
                [12, 20],
                [9, 21],
                [9, 19],
                [10, 22],
                [10, 18],
                [11, 21],
                [11, 19]
            ]
        );

        assert_eq!(
            WaterCycleSystem::circle(1, 20, 2, 100),
            vec![
                [3, 20],
                [0, 21],
                [0, 19],
                [1, 22],
                [1, 18],
                [2, 21],
                [2, 19]
            ]
        );

        assert_eq!(
            WaterCycleSystem::circle(0, 20, 2, 100),
            vec![[2, 20], [0, 22], [0, 18], [1, 21], [1, 19]]
        );

        assert_eq!(
            WaterCycleSystem::circle(98, 20, 2, 100),
            vec![
                [96, 20],
                [97, 21],
                [97, 19],
                [98, 22],
                [98, 18],
                [99, 21],
                [99, 19]
            ]
        );

        assert_eq!(
            WaterCycleSystem::circle(99, 20, 2, 100),
            vec![[97, 20], [98, 21], [98, 19], [99, 22], [99, 18],]
        );
    }
}
