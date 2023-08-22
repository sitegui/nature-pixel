use crate::cell::cell_animal::CellAnimal;
use crate::config::Config;
use crate::map::Map;
use crate::monitored_rwlock::MonitoredRwLock;
use crate::point::Point;
use itertools::Itertools;
use rand::prelude::{IteratorRandom, SliceRandom, SmallRng};
use rand::{Rng, SeedableRng};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time;

#[derive(Debug)]
pub struct Snake {
    species: SnakeSpecies,
    segment: Option<SnakeSegment>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SnakeSpecies {
    A,
    B,
    C,
}

#[derive(Debug, Clone, Copy)]
struct SnakeSegment {
    kind: SnakeSegmentKind,
    next_segment: Option<Point>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum SnakeSegmentKind {
    /// The head is a special segment because it's used to actually identify each individual.
    /// Without this distinction, if individuals came too close to each other and had their segments
    /// well aligned, they would be re-interpreted as one.
    Head {
        last_feeding: Instant,
    },
    Body,
}

#[derive(Debug)]
pub struct SnakeSystem {
    map: Arc<MonitoredRwLock<Map>>,
    rng: SmallRng,
    a_max_size: usize,
    a_move_ratio: f64,
    b_max_size: usize,
    b_move_ratio: f64,
    c_max_size: usize,
    c_move_ratio: f64,
    min_size: usize,
    eating_radius: usize,
    starvation_delay: Duration,
    tick_sleep: Duration,
}

#[derive(Debug)]
enum Change {
    NewSnake(Vec<Point>),
    Starve(Vec<Point>),
    Move {
        snake: Vec<Point>,
        target: Point,
    },
    Eat {
        head: Point,
        new_head: Point,
        food: Point,
    },
    Death(Point),
}

#[derive(Debug, Default)]
struct SnakeSegmentSet {
    heads: HashMap<Point, SnakeSegment>,
    bodies: HashMap<Point, SnakeSegment>,
    spare_parts: HashSet<Point>,
}

impl SnakeSystem {
    pub fn new(config: &Config, map: Arc<MonitoredRwLock<Map>>) -> Self {
        Self {
            map,
            rng: SmallRng::from_entropy(),
            a_max_size: config.snake_a_max_size,
            a_move_ratio: config.snake_a_move_ratio,
            b_max_size: config.snake_b_max_size,
            b_move_ratio: config.snake_b_move_ratio,
            c_max_size: config.snake_c_max_size,
            c_move_ratio: config.snake_c_move_ratio,
            min_size: config.snake_min_size,
            eating_radius: config.snake_eating_radius,
            starvation_delay: Duration::from_secs(config.snake_starvation_delay_seconds),
            tick_sleep: Duration::from_secs(config.snake_tick_seconds),
        }
    }

    pub async fn run(mut self) {
        loop {
            let changes = self.determine_changes();
            self.apply_changes(changes);
            time::sleep(self.tick_sleep).await;
        }
    }

    fn determine_changes(&mut self) -> Vec<Change> {
        let mut changes = Vec::new();
        let map = self.map.clone();
        let map = map.read(module_path!());

        // Index the snakes by their species, kind and position.
        // Also index amphibians, their preys.
        let mut snakes: HashMap<SnakeSpecies, SnakeSegmentSet> = HashMap::default();
        let mut uneaten_preys = HashSet::new();
        for (ij, cell) in map.cells().indexed_iter() {
            match cell.animal() {
                CellAnimal::Snake(snake) => {
                    let point = Point::new_ij(ij);
                    let segment_set = snakes.entry(snake.species).or_default();

                    match snake.segment {
                        None => {
                            segment_set.spare_parts.insert(point);
                        }
                        Some(segment) => {
                            let kind = match segment.kind {
                                SnakeSegmentKind::Head { .. } => &mut segment_set.heads,
                                SnakeSegmentKind::Body => &mut segment_set.bodies,
                            };

                            kind.insert(point, segment);
                        }
                    }
                }
                CellAnimal::Amphibian(_) => {
                    uneaten_preys.insert(Point::new_ij(ij));
                }
                _ => {}
            }
        }

        // Determine the changes for each species
        for (species, segment_set) in snakes {
            changes.extend(self.determine_species_changes(
                &map,
                species,
                segment_set,
                &mut uneaten_preys,
            ));
        }

        changes
    }

    fn determine_species_changes(
        &mut self,
        map: &Map,
        species: SnakeSpecies,
        mut segment_set: SnakeSegmentSet,
        uneaten_preys: &mut HashSet<Point>,
    ) -> Vec<Change> {
        let max_size = self.max_size(species);
        let mut changes = Vec::new();
        let now = Instant::now();

        // Determine where to move to each existing snake
        for (point, head) in segment_set.heads {
            let snake_points =
                self.extract_snake(max_size, point, head.next_segment, &mut segment_set.bodies);

            match snake_points {
                None => {
                    // This snake is now invalid and dies
                    changes.push(Change::Death(point));
                }
                Some(snake_points) => {
                    let change =
                        self.determine_starvation(now, head, &snake_points)
                            .or_else(|| {
                                self.determine_next_movement(
                                    map,
                                    species,
                                    snake_points,
                                    uneaten_preys,
                                    max_size,
                                )
                            });

                    if let Some(change) = change {
                        changes.push(change);
                    }
                }
            }
        }

        // Dangling bodies die
        for point in segment_set.bodies.into_keys() {
            changes.push(Change::Death(point));
        }

        // Detect new snakes
        let spare_parts = &mut segment_set.spare_parts;
        while let Some(&point) = spare_parts.iter().next() {
            spare_parts.remove(&point);
            if let Some(snake_points) = self.determine_new_snake(point, spare_parts) {
                changes.push(Change::NewSnake(snake_points));
            }
        }

        changes
    }

    fn apply_changes(&self, changes: Vec<Change>) {
        let mut map = self.map.write(module_path!());
        let mut changed_map = false;

        for change in changes {
            match change {
                Change::NewSnake(points) => {
                    self.apply_new_snake(&mut map, &points);
                }
                Change::Move { snake, target } => {
                    self.apply_move(&mut map, &snake, target);
                    changed_map = true;
                }
                Change::Eat {
                    head,
                    new_head,
                    food,
                } => {
                    self.apply_eat(&mut map, head, new_head, food);
                    changed_map = true;
                }
                Change::Death(point) => {
                    self.apply_death(&mut map, point);
                    changed_map = true;
                }
                Change::Starve(points) => {
                    self.apply_starvation(&mut map, points);
                    changed_map = true;
                }
            }
        }

        if changed_map {
            map.notify_update();
        }
    }

    /// Find a new snake that contains the given `point`. The snake orientation will be randomly
    /// chosen. Also, if there are multiple ambiguous snake formations, the result will be randomly
    /// determined.
    ///
    /// The returned segments are ordered such that the head is the first element. Note that the
    /// point passed as argument does not need to be the actual head nor tail.
    fn determine_new_snake(
        &mut self,
        point: Point,
        spare_parts: &mut HashSet<Point>,
    ) -> Option<Vec<Point>> {
        let snake_size = self.min_size;
        let mut segments = VecDeque::with_capacity(snake_size);

        let mut head = point;
        let mut tail = point;
        segments.push_front(point);

        let mut candidates = Vec::with_capacity(8);
        while segments.len() < snake_size {
            candidates.clear();
            for (new_head, base) in [(true, head), (false, tail)] {
                for direction in Point::DIRECTIONS {
                    let candidate = base + direction;

                    if spare_parts.contains(&candidate) {
                        candidates.push((new_head, candidate));
                    }
                }
            }

            match candidates.choose(&mut self.rng) {
                None => break,
                Some(&(new_head, candidate)) => {
                    if new_head {
                        head = candidate;
                        segments.push_front(head);
                    } else {
                        tail = candidate;
                        segments.push_back(tail);
                    }
                    spare_parts.remove(&candidate);
                }
            }
        }

        (segments.len() == snake_size).then(|| segments.into())
    }

    /// Find all the points of the snake beginning at a given head. It will only return the snake if
    /// it is big enough.
    ///
    /// This will also remove all the referenced body segments from the set, so that they cannot be
    /// used as part of another snake.
    fn extract_snake(
        &self,
        max_size: usize,
        head: Point,
        head_next_segment: Option<Point>,
        body_segment_set: &mut HashMap<Point, SnakeSegment>,
    ) -> Option<Vec<Point>> {
        let mut points = Vec::with_capacity(max_size);
        points.push(head);
        let mut next_segment = head_next_segment;

        while let (Some(target), true) = (next_segment, points.len() < max_size) {
            match body_segment_set.remove(&target) {
                None => break,
                Some(snake_segment) => {
                    points.push(target);
                    next_segment = snake_segment.next_segment;
                }
            }
        }

        (points.len() >= self.min_size).then_some(points)
    }

    /// Check if the snake is starving
    fn determine_starvation(
        &self,
        now: Instant,
        head: SnakeSegment,
        snake_points: &[Point],
    ) -> Option<Change> {
        if let SnakeSegmentKind::Head { last_feeding } = head.kind {
            if now - last_feeding > self.starvation_delay {
                return Some(Change::Starve(snake_points.to_vec()));
            }
        }

        None
    }

    /// Determine where the snake should next move to
    fn determine_next_movement(
        &mut self,
        map: &Map,
        species: SnakeSpecies,
        snake_points: Vec<Point>,
        uneaten_preys: &mut HashSet<Point>,
        max_size: usize,
    ) -> Option<Change> {
        if !self.rng.gen_bool(self.move_ratio(species)) {
            return None;
        }

        if snake_points.len() < max_size {
            // Find a prey to eat
            let head = snake_points[0];
            if let Some(change) = self.determine_eat_nearby_prey(map, uneaten_preys, head) {
                return Some(change);
            }

            // Find the closest prey
            let closest_preys = uneaten_preys
                .iter()
                .copied()
                .min_set_by_key(|prey| prey.distance(head));
            if let Some(prey) = closest_preys.choose(&mut self.rng).copied() {
                let target = self.find_movement_target(map, head, prey)?;

                return Some(Change::Move {
                    snake: snake_points,
                    target,
                });
            }
        }

        self.determine_random_walk(map, snake_points)
    }

    /// Determine if can eat a nearby prey
    fn determine_eat_nearby_prey(
        &mut self,
        map: &Map,
        uneaten_preys: &mut HashSet<Point>,
        head: Point,
    ) -> Option<Change> {
        let food = head
            .circle(self.eating_radius, map.size())
            .filter(|target| uneaten_preys.contains(target))
            .choose(&mut self.rng)?;

        let new_head = self.find_movement_target(map, head, food)?;
        uneaten_preys.remove(&food);
        Some(Change::Eat {
            head,
            food,
            new_head,
        })
    }

    fn determine_random_walk(&mut self, map: &Map, snake_points: Vec<Point>) -> Option<Change> {
        let head = snake_points[0];
        let directions = if snake_points.len() == 1 {
            vec![
                (head + Point::X, 1.0),
                (head + Point::Y, 1.0),
                (head - Point::X, 1.0),
                (head - Point::Y, 1.0),
            ]
        } else {
            let forward = snake_points[0] - snake_points[1];
            vec![
                (head + forward, 4.0),
                (head + forward.turn_right(), 1.0),
                (head + forward.turn_left(), 1.0),
            ]
        };
        let valid_targets = directions
            .into_iter()
            .filter(|&(target, _)| {
                map.cells()
                    .get(target)
                    .map(|cell| cell.animal().is_empty())
                    .unwrap_or(false)
            })
            .collect_vec();

        valid_targets
            .choose_weighted(&mut self.rng, |&(_, weight)| weight)
            .ok()
            .map(|&(target, _)| Change::Move {
                snake: snake_points,
                target,
            })
    }

    /// Find a valid movement that gets the snake closer to the given goal
    fn find_movement_target(&mut self, map: &Map, head: Point, goal: Point) -> Option<Point> {
        let best_moves = Point::DIRECTIONS
            .into_iter()
            .map(|direction| head + direction)
            .filter(|&target| {
                map.cells()
                    .get(target)
                    .map(|cell| cell.animal().is_empty())
                    .unwrap_or(false)
            })
            .min_set_by_key(|target| target.distance(goal));

        let target = best_moves.choose(&mut self.rng).copied();
        tracing::debug!(
            "find_movement_target for {:?} towards {:?} = {:?}",
            head,
            goal,
            target
        );
        target
    }

    fn max_size(&self, species: SnakeSpecies) -> usize {
        match species {
            SnakeSpecies::A => self.a_max_size,
            SnakeSpecies::B => self.b_max_size,
            SnakeSpecies::C => self.c_max_size,
        }
    }

    fn move_ratio(&self, species: SnakeSpecies) -> f64 {
        match species {
            SnakeSpecies::A => self.a_move_ratio,
            SnakeSpecies::B => self.b_move_ratio,
            SnakeSpecies::C => self.c_move_ratio,
        }
    }

    fn apply_new_snake(&self, map: &mut Map, points: &[Point]) {
        let Some(head) = map.cells()[points[0]].animal().snake() else {return};
        let species = head.species;

        // Check invariants for all cells: same snake species and free segment
        let is_valid_snake = points
            .iter()
            .map(|&point| map.cells()[point].animal().snake())
            .all(|snake| match snake {
                None => false,
                Some(snake) => snake.species == species && snake.segment.is_none(),
            });

        if !is_valid_snake {
            return;
        }

        for (i, &point) in points.iter().enumerate() {
            if let Some(snake) = map.cells_mut()[point].animal_mut().snake_mut() {
                let kind = if i == 0 {
                    SnakeSegmentKind::Head {
                        last_feeding: Instant::now(),
                    }
                } else {
                    SnakeSegmentKind::Body
                };
                snake.segment = Some(SnakeSegment {
                    kind,
                    next_segment: points.get(i + 1).copied(),
                });
            }
        }
    }

    fn apply_move(&self, map: &mut Map, snake: &[Point], target_point: Point) {
        // Check head is valid
        let Some(head) = map.cells()[snake[0]].animal().snake() else {return};
        let species = head.species;
        let last_feeding = match head.segment {
            None => return,
            Some(head_segment) => {
                if head_segment.next_segment != snake.get(1).copied() {
                    return;
                }

                match head_segment.kind {
                    SnakeSegmentKind::Body => return,
                    SnakeSegmentKind::Head { last_feeding } => last_feeding,
                }
            }
        };

        // Check snake body is valid
        for (i, &point) in snake.iter().enumerate().skip(1) {
            match map.cells()[point].animal().snake() {
                None => return,
                Some(cell) => {
                    if cell.species != species
                        || !cell.is_body()
                        || cell.next_segment() != snake.get(i + 1).copied()
                    {
                        return;
                    }
                }
            }
        }

        // Check target is valid
        let target = map.cells_mut()[target_point].animal_mut();
        if !target.is_empty() {
            return;
        }

        // Update cells
        *target = CellAnimal::Snake(Box::new(Snake {
            species,
            segment: Some(SnakeSegment {
                kind: SnakeSegmentKind::Head { last_feeding },
                next_segment: Some(snake[0]),
            }),
        }));
        if let Some(head) = map.cells_mut()[snake[0]].animal_mut().snake_mut() {
            if let Some(head) = &mut head.segment {
                head.kind = SnakeSegmentKind::Body;
            }
        }
        let tail = map.cells_mut()[snake[snake.len() - 1]].animal_mut();
        *tail = CellAnimal::Empty;
        let new_tail = if snake.len() == 1 {
            target_point
        } else {
            snake[snake.len() - 2]
        };
        if let Some(new_tail) = map.cells_mut()[new_tail].animal_mut().snake_mut() {
            if let Some(new_tail) = &mut new_tail.segment {
                new_tail.next_segment = None;
            }
        }
    }

    fn apply_eat(
        &self,
        map: &mut Map,
        head_point: Point,
        new_head_point: Point,
        food_point: Point,
    ) {
        let (head, new_head, food) = map.three_cells_mut(head_point, new_head_point, food_point);

        let Some(head) = head.animal_mut().snake_mut() else {return};
        let new_head = new_head.animal_mut();
        let food = food.animal_mut();

        let Some(head_segment) = &mut head.segment else {return};
        if !matches!(head_segment.kind, SnakeSegmentKind::Head { .. }) {
            return;
        }

        if !new_head.is_empty() {
            return;
        }

        if food.amphibian().is_none() {
            return;
        }

        head_segment.kind = SnakeSegmentKind::Body;
        *new_head = CellAnimal::Snake(Box::new(Snake {
            species: head.species,
            segment: Some(SnakeSegment {
                kind: SnakeSegmentKind::Head {
                    last_feeding: Instant::now(),
                },
                next_segment: Some(head_point),
            }),
        }));
        *food = CellAnimal::Empty;
    }

    fn apply_death(&self, map: &mut Map, point: Point) {
        let cell = map.cells_mut()[point].animal_mut();

        if cell.snake().is_none() {
            return;
        }

        *cell = CellAnimal::Dead;
    }

    fn apply_starvation(&self, map: &mut Map, points: Vec<Point>) {
        for point in points {
            self.apply_death(map, point);
        }
    }
}

impl Snake {
    pub fn new(species: SnakeSpecies) -> Self {
        Snake {
            species,
            segment: None,
        }
    }

    pub fn species(&self) -> SnakeSpecies {
        self.species
    }

    fn is_body(&self) -> bool {
        self.segment
            .map(|segment| segment.kind == SnakeSegmentKind::Body)
            .unwrap_or(false)
    }

    fn next_segment(&self) -> Option<Point> {
        self.segment.and_then(|segment| segment.next_segment)
    }
}
