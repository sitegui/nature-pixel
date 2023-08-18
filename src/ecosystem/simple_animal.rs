use crate::cell::cell_animal::CellAnimal;
use crate::cell::Cell;
use crate::map::Map;
use crate::monitored_rwlock::MonitoredRwLock;
use crate::point::Point;
use itertools::Itertools;
use rand::prelude::{IteratorRandom, SliceRandom, SmallRng};
use rand::SeedableRng;
use std::marker::PhantomData;
use std::mem;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time;

#[derive(Debug)]
pub struct SimpleAnimal {
    state: SimpleAnimalState,
    direction: Point,
    destination: Option<Point>,
    last_feeding: Instant,
}

#[derive(Debug)]
pub struct SimpleAnimalSystem<K> {
    map: Arc<MonitoredRwLock<Map>>,
    tick_sleep: Duration,
    eating_radius: usize,
    mating_radius: usize,
    destination_radius: usize,
    rng: SmallRng,
    _phantom: PhantomData<K>,
    starvation_delay: Duration,
}

#[derive(Debug, Copy, Clone)]
pub struct WalkCandidate {
    target: Point,
    new_direction: Point,
}

impl WalkCandidate {
    pub fn new(point: Point, direction: Point, scale: usize) -> Self {
        WalkCandidate {
            target: point + direction * scale,
            new_direction: direction,
        }
    }
}

pub trait SimpleAnimalKind {
    type WalkCandidates: Iterator<Item = WalkCandidate>;
    fn get(cell: &Cell) -> Option<&SimpleAnimal>;
    fn get_mut(cell: &mut Cell) -> Option<&mut SimpleAnimal>;
    fn walk_candidates(point: Point, direction: Point) -> Self::WalkCandidates;
    fn is_food_goal(cell: &Cell) -> bool;
    fn is_mating_ground_goal(cell: &Cell) -> bool;
    fn build_cell(simple_animal: SimpleAnimal) -> CellAnimal;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[allow(clippy::enum_variant_names)]
enum SimpleAnimalState {
    SearchFood,
    SearchMatingGround,
    SearchPartner,
}

#[derive(Debug)]
enum Change {
    Eat(Point),
    SearchPartner,
    Mate { partner: Point, new_born: Point },
    MoveTo(WalkCandidate),
    SetDestination(Point),
    SearchMatingGround,
    Starve,
}

impl Default for SimpleAnimal {
    fn default() -> Self {
        SimpleAnimal {
            state: SimpleAnimalState::SearchFood,
            direction: Point::X,
            destination: None,
            last_feeding: Instant::now(),
        }
    }
}

impl<K: SimpleAnimalKind> SimpleAnimalSystem<K> {
    pub fn new(
        tick_sleep: Duration,
        eating_radius: usize,
        mating_radius: usize,
        destination_radius: usize,
        starvation_delay: Duration,
        map: Arc<MonitoredRwLock<Map>>,
    ) -> Self {
        Self {
            map,
            tick_sleep,
            eating_radius,
            mating_radius,
            destination_radius,
            starvation_delay,
            rng: SmallRng::from_entropy(),
            _phantom: PhantomData,
        }
    }

    pub async fn run(mut self) {
        loop {
            let changes = self.determine_changes();
            self.apply_changes(changes);
            time::sleep(self.tick_sleep).await;
        }
    }

    /// Determine what should change for each simple animal
    fn determine_changes(&mut self) -> Vec<(Point, Change)> {
        let now = Instant::now();
        let mut changes = Vec::new();
        let map = self.map.read(module_path!());

        for (ij, cell) in map.cells().indexed_iter() {
            if let Some(simple_animal) = K::get(cell) {
                let point = Point::new_ij(ij);
                let change = Self::determine_starvation(self.starvation_delay, now, simple_animal)
                    .or_else(|| {
                        Self::determine_reached_goal(
                            &mut self.rng,
                            &map,
                            point,
                            simple_animal,
                            self.eating_radius,
                            self.mating_radius,
                        )
                    })
                    .or_else(|| {
                        Self::determine_next_walk(&mut self.rng, &map, point, simple_animal)
                    })
                    .unwrap_or_else(|| {
                        Self::determine_next_destination(
                            &mut self.rng,
                            &map,
                            point,
                            simple_animal,
                            self.destination_radius,
                        )
                    });
                changes.push((point, change));
            }
        }

        changes
    }

    /// Check if the animal is starving
    fn determine_starvation(
        starvation_delay: Duration,
        now: Instant,
        simple_animal: &SimpleAnimal,
    ) -> Option<Change> {
        (now - simple_animal.last_feeding > starvation_delay).then_some(Change::Starve)
    }

    /// Check if the goal of the current state was met
    fn determine_reached_goal(
        rng: &mut SmallRng,
        map: &Map,
        point: Point,
        simple_animal: &SimpleAnimal,
        eating_radius: usize,
        mating_radius: usize,
    ) -> Option<Change> {
        match simple_animal.state {
            SimpleAnimalState::SearchFood => point
                .circle(eating_radius, map.size())
                .find(|&target| Self::check_food_goal(map, target))
                .map(Change::Eat),
            SimpleAnimalState::SearchMatingGround => point
                .surroundings()
                .into_iter()
                .find(|&target| Self::check_mating_ground_goal(map, target))
                .map(|_| Change::SearchPartner),
            SimpleAnimalState::SearchPartner => point
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
        simple_animal: &SimpleAnimal,
    ) -> Option<Change> {
        let destination = simple_animal.destination?;
        if destination == point {
            return None;
        }

        let closest_candidates = K::walk_candidates(point, simple_animal.direction)
            .filter(|candidate| {
                map.cells()
                    .get(candidate.target)
                    .map(|cell| cell.animal().is_empty())
                    .unwrap_or(false)
            })
            .min_set_by_key(|candidate| candidate.target.distance(destination));

        closest_candidates.choose(rng).copied().map(Change::MoveTo)
    }

    /// Determine a next walking destination, trying to achieve this state's goal
    fn determine_next_destination(
        rng: &mut SmallRng,
        map: &Map,
        point: Point,
        simple_animal: &SimpleAnimal,
        destination_radius: usize,
    ) -> Change {
        let search_circle = point.circle(destination_radius, map.size());

        // Find a random point that fulfil the goal
        let goal_destination = match simple_animal.state {
            SimpleAnimalState::SearchFood => search_circle
                .filter(|&target| Self::check_food_goal(map, target))
                .choose(rng),
            SimpleAnimalState::SearchMatingGround => search_circle
                .filter(|&target| Self::check_mating_ground_goal(map, target))
                .choose(rng),
            SimpleAnimalState::SearchPartner => search_circle
                .filter(|&target| Self::check_partner_goal(map, point, target))
                .choose(rng),
        };
        if let Some(goal_destination) = goal_destination {
            return Change::SetDestination(goal_destination);
        }

        match simple_animal.state {
            // If no direct goal-fulfilling destination was found, walk to a random far point
            SimpleAnimalState::SearchFood | SimpleAnimalState::SearchMatingGround => {
                let destination = point
                    .circumference(destination_radius, map.size())
                    .choose(rng)
                    .expect("must have at least one point");

                Change::SetDestination(destination)
            }
            // If no direct goal-fulfilling destination was found, walk to a random far point that
            // is still a mating ground
            SimpleAnimalState::SearchPartner => point
                .circle(destination_radius, map.size())
                .filter(|&target| Self::check_mating_ground_goal(map, target))
                .max_set_by_key(|target| target.distance(point))
                .choose(rng)
                .copied()
                .map(Change::SetDestination)
                .unwrap_or(Change::SearchMatingGround),
        }
    }

    fn check_food_goal(map: &Map, point: Point) -> bool {
        map.cells().get(point).map(K::is_food_goal).unwrap_or(false)
    }

    fn check_mating_ground_goal(map: &Map, point: Point) -> bool {
        map.cells()
            .get(point)
            .map(K::is_mating_ground_goal)
            .unwrap_or(false)
    }

    fn check_partner_goal(map: &Map, self_point: Point, point: Point) -> bool {
        if self_point == point {
            return false;
        }

        map.cells()
            .get(point)
            .and_then(|cell| K::get(cell))
            .map(|partner| partner.state == SimpleAnimalState::SearchPartner)
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
        let now = Instant::now();
        let mut map = self.map.write(module_path!());
        let mut changed_map = false;

        for (point, change) in changes {
            tracing::debug!("{:?}: apply {:?}", point, change);

            match change {
                Change::Starve => {
                    let cell = &mut map.cells_mut()[point];
                    if let Some(simple_animal) = K::get_mut(cell) {
                        match simple_animal.state {
                            SimpleAnimalState::SearchFood => {
                                *cell.animal_mut() = CellAnimal::Dead;
                            }
                            SimpleAnimalState::SearchMatingGround
                            | SimpleAnimalState::SearchPartner => {
                                simple_animal.state = SimpleAnimalState::SearchFood;
                                simple_animal.last_feeding = now;
                                simple_animal.destination = None;
                            }
                        }
                    }
                }
                Change::SearchMatingGround => {
                    let cell = &mut map.cells_mut()[point];
                    if let Some(simple_animal) = K::get_mut(cell) {
                        if simple_animal.state != SimpleAnimalState::SearchMatingGround {
                            simple_animal.state = SimpleAnimalState::SearchMatingGround;
                            simple_animal.destination = None;
                        }
                    }
                }
                Change::SearchPartner => {
                    let cell = &mut map.cells_mut()[point];
                    if let Some(simple_animal) = K::get_mut(cell) {
                        if simple_animal.state == SimpleAnimalState::SearchMatingGround {
                            simple_animal.state = SimpleAnimalState::SearchPartner;
                            simple_animal.destination = None;
                        }
                    }
                }
                Change::SetDestination(destination) => {
                    let cell = &mut map.cells_mut()[point];
                    if let Some(simple_animal) = K::get_mut(cell) {
                        simple_animal.destination = Some(destination);
                    }
                }
                Change::Eat(target) => {
                    let (animal, food) = map.two_cells_mut(point, target);

                    if let (Some(simple_animal), true) = (K::get_mut(animal), K::is_food_goal(food))
                    {
                        if simple_animal.state == SimpleAnimalState::SearchFood {
                            simple_animal.state = SimpleAnimalState::SearchMatingGround;
                            simple_animal.destination = None;
                            simple_animal.last_feeding = now;
                            *food.animal_mut() = CellAnimal::Empty;
                            changed_map = true;
                        }
                    }
                }
                Change::Mate { partner, new_born } => {
                    let (partner_1, partner_2, new_born) =
                        map.three_cells_mut(point, partner, new_born);

                    if let (Some(partner_1), Some(partner_2), true) = (
                        K::get_mut(partner_1),
                        K::get_mut(partner_2),
                        new_born.animal().is_empty(),
                    ) {
                        if partner_1.state == SimpleAnimalState::SearchPartner
                            && partner_2.state == SimpleAnimalState::SearchPartner
                        {
                            partner_1.state = SimpleAnimalState::SearchFood;
                            partner_2.state = SimpleAnimalState::SearchFood;
                            *new_born.animal_mut() = K::build_cell(SimpleAnimal::default());
                            changed_map = true;
                        }
                    }
                }
                Change::MoveTo(candidate) => {
                    let (from, to) = map.two_cells_mut(point, candidate.target);

                    if let (Some(from_insect), true) = (K::get_mut(from), to.animal().is_empty()) {
                        if from_insect.destination == Some(candidate.target) {
                            from_insect.destination = None;
                        }

                        from_insect.direction = candidate.new_direction;
                        mem::swap(from.animal_mut(), to.animal_mut());
                        changed_map = true;
                    }
                }
            }
        }

        if changed_map {
            map.notify_update();
        }
    }
}
