use crate::cell::cell_animal::CellAnimal;
use crate::config::Config;
use crate::map::Map;
use crate::point::Point;
use itertools::Itertools;
use rand::prelude::{IteratorRandom, SliceRandom, SmallRng};
use rand::SeedableRng;
use std::mem;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time;

#[derive(Debug)]
pub struct Insect {
    state: InsectState,
    direction: Point,
    destination: Option<Point>,
}

#[derive(Debug)]
pub struct InsectSystem {
    map: Arc<RwLock<Map>>,
    tick_sleep: Duration,
    mating_radius: usize,
    destination_radius: usize,
    rng: SmallRng,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[allow(clippy::enum_variant_names)]
enum InsectState {
    SearchFood,
    SearchGrass,
    SearchPartner,
}

#[derive(Debug)]
enum Change {
    Eat(Point),
    SearchPartner,
    Mate { partner: Point, new_born: Point },
    MoveTo(Point),
    SetDestination(Point),
}

impl Default for Insect {
    fn default() -> Self {
        Insect {
            state: InsectState::SearchFood,
            direction: Point::X,
            destination: None,
        }
    }
}

impl InsectSystem {
    pub fn new(config: &Config, map: Arc<RwLock<Map>>) -> Self {
        Self {
            map,
            tick_sleep: Duration::from_secs(config.insect_tick_seconds),
            mating_radius: config.insect_mating_radius,
            destination_radius: config.insect_destination_radius,
            rng: SmallRng::from_entropy(),
        }
    }

    pub async fn run(mut self) {
        loop {
            let changes = self.determine_changes();
            self.apply_changes(changes);
            time::sleep(self.tick_sleep).await;
        }
    }

    /// Determine what should change for each insect
    fn determine_changes(&mut self) -> Vec<(Point, Change)> {
        let mut changes = Vec::new();
        let map = self.map.read().unwrap();

        for (ij, cell) in map.cells().indexed_iter() {
            if let Some(insect) = cell.animal().insect() {
                let point = Point::new_ij(ij);
                let change = Self::determine_reached_goal(
                    &mut self.rng,
                    &map,
                    point,
                    insect,
                    self.mating_radius,
                )
                .or_else(|| Self::determine_next_walk(&mut self.rng, &map, point, insect))
                .unwrap_or_else(|| {
                    Self::determine_next_destination(
                        &mut self.rng,
                        &map,
                        point,
                        insect,
                        self.destination_radius,
                    )
                });
                changes.push((point, change));
            }
        }

        changes
    }

    /// Check if the goal of the current state was met
    fn determine_reached_goal(
        rng: &mut SmallRng,
        map: &Map,
        point: Point,
        insect: &Insect,
        mating_radius: usize,
    ) -> Option<Change> {
        match insect.state {
            InsectState::SearchFood => point
                .surroundings()
                .into_iter()
                .find(|&target| Self::check_food_goal(map, target))
                .map(Change::Eat),
            InsectState::SearchGrass => point
                .surroundings()
                .into_iter()
                .find(|&target| Self::check_grass_goal(map, target))
                .map(|_| Change::SearchPartner),
            InsectState::SearchPartner => point
                .circle(mating_radius, map.size())
                .find(|&target| Self::check_partner_goal(map, point, target))
                .and_then(|partner| {
                    point
                        .circle(mating_radius, map.size())
                        .filter(|&target| Self::check_new_born(map, target))
                        .choose(rng)
                        .map(|new_born| Change::Mate { partner, new_born })
                }),
        }
    }

    /// Determine where to walk to next
    fn determine_next_walk(
        rng: &mut SmallRng,
        map: &Map,
        point: Point,
        insect: &Insect,
    ) -> Option<Change> {
        let destination = insect.destination?;
        if destination == point {
            return None;
        }

        let candidates = [
            point + insect.direction.turn_right(),
            point + insect.direction.turn_left(),
            point + insect.direction.turn_over(),
        ];

        candidates
            .into_iter()
            .filter(|&target| {
                map.cells()
                    .get(target)
                    .map(|cell| cell.animal().is_empty())
                    .unwrap_or(false)
            })
            .min_set_by_key(|target| target.distance(destination))
            .choose(rng)
            .copied()
            .map(Change::MoveTo)
    }

    /// Determine a next walking destination, trying to achieve this state's goal
    fn determine_next_destination(
        rng: &mut SmallRng,
        map: &Map,
        point: Point,
        insect: &Insect,
        destination_radius: usize,
    ) -> Change {
        let search_circle = point.circle(destination_radius, map.size());

        // Find a random point that fulfil the goal
        let goal_destination = match insect.state {
            InsectState::SearchFood => search_circle
                .filter(|&target| Self::check_food_goal(map, target))
                .choose(rng),
            InsectState::SearchGrass => search_circle
                .filter(|&target| Self::check_grass_goal(map, target))
                .choose(rng),
            InsectState::SearchPartner => search_circle
                .filter(|&target| Self::check_partner_goal(map, point, target))
                .choose(rng),
        };
        if let Some(goal_destination) = goal_destination {
            return Change::SetDestination(goal_destination);
        }

        // If no direct goal-fulfilling destination was found, walk to a random far point
        let destination = point
            .circumference(destination_radius, map.size())
            .choose(rng)
            .expect("must have at least one point");
        Change::SetDestination(destination)
    }

    fn check_food_goal(map: &Map, point: Point) -> bool {
        map.cells()
            .get(point)
            .map(|cell| cell.animal().is_dead())
            .unwrap_or(false)
    }

    fn check_grass_goal(map: &Map, point: Point) -> bool {
        map.cells()
            .get(point)
            .map(|cell| !cell.grass().is_empty())
            .unwrap_or(false)
    }

    fn check_partner_goal(map: &Map, self_point: Point, point: Point) -> bool {
        if self_point == point {
            return false;
        }

        map.cells()
            .get(point)
            .and_then(|cell| cell.animal().insect())
            .map(|partner| partner.state == InsectState::SearchPartner)
            .unwrap_or(false)
    }

    fn check_new_born(map: &Map, point: Point) -> bool {
        map.cells()
            .get(point)
            .map(|cell| cell.animal().is_empty())
            .unwrap_or(false)
    }

    /// Apply the changes, taking care to re-check if the necessary conditions still hold
    fn apply_changes(&mut self, changes: Vec<(Point, Change)>) {
        let mut map = self.map.write().unwrap();
        let mut changed = false;

        for (point, change) in changes {
            tracing::debug!("{:?}: apply {:?}", point, change);
            match change {
                Change::SearchPartner => {
                    if let Some(insect) = map.cells_mut()[point].animal_mut().insect_mut() {
                        if insect.state == InsectState::SearchGrass {
                            insect.state = InsectState::SearchPartner;
                            insect.destination = None;
                            changed = true;
                        }
                    }
                }
                Change::SetDestination(destination) => {
                    if let Some(insect) = map.cells_mut()[point].animal_mut().insect_mut() {
                        insect.destination = Some(destination);
                        changed = true;
                    }
                }
                Change::Eat(target) => {
                    let (insect, food) = map.two_cells_mut(point, target);

                    if let (Some(insect), true) =
                        (insect.animal_mut().insect_mut(), food.animal().is_dead())
                    {
                        if insect.state == InsectState::SearchFood {
                            insect.state = InsectState::SearchGrass;
                            insect.destination = None;
                            *food.animal_mut() = CellAnimal::Empty;
                            changed = true;
                        }
                    }
                }
                Change::Mate { partner, new_born } => {
                    let (partner_1, partner_2, new_born) =
                        map.three_cells_mut(point, partner, new_born);

                    if let (Some(partner_1), Some(partner_2), true) = (
                        partner_1.animal_mut().insect_mut(),
                        partner_2.animal_mut().insect_mut(),
                        new_born.animal().is_empty(),
                    ) {
                        if partner_1.state == InsectState::SearchPartner
                            && partner_2.state == InsectState::SearchPartner
                        {
                            partner_1.state = InsectState::SearchFood;
                            partner_2.state = InsectState::SearchFood;
                            *new_born.animal_mut() = CellAnimal::Insect(Default::default());
                            changed = true;
                        }
                    }
                }
                Change::MoveTo(target) => {
                    let (from, to) = map.two_cells_mut(point, target);

                    if let (Some(from_insect), true) =
                        (from.animal_mut().insect_mut(), to.animal().is_empty())
                    {
                        if from_insect.destination == Some(target) {
                            from_insect.destination = None;
                        }

                        from_insect.direction = target - point;
                        mem::swap(from.animal_mut(), to.animal_mut());
                        changed = true;
                    }
                }
            }
        }

        if changed {
            map.notify_update();
        }
    }
}
