use crate::cell::cell_animal::CellAnimal;
use crate::config::Config;
use crate::map::Map;
use crate::monitored_rwlock::MonitoredRwLock;
use crate::point::Point;
use rand::prelude::{SliceRandom, SmallRng};
use rand::SeedableRng;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::Duration;
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
    Head,
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
    prey_radius: usize,
    starvation_delay: Duration,
    tick_sleep: Duration,
}

#[derive(Debug)]
enum Change {
    NewSnake(Vec<Point>),
    Move {
        head: Point,
        tail: Point,
        target: Point,
    },
    Eat {
        head: Point,
        target: Point,
    },
    Die(Point),
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
            prey_radius: config.snake_prey_radius,
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

        // Index the snakes by their species, kind and position
        let mut snakes: HashMap<SnakeSpecies, SnakeSegmentSet> = HashMap::default();
        for (ij, cell) in map.cells().indexed_iter() {
            if let CellAnimal::Snake(snake) = cell.animal() {
                let point = Point::new_ij(ij);
                let segment_set = snakes.entry(snake.species).or_default();

                match snake.segment {
                    None => {
                        segment_set.spare_parts.insert(point);
                    }
                    Some(segment) => {
                        let kind = match segment.kind {
                            SnakeSegmentKind::Head => &mut segment_set.heads,
                            SnakeSegmentKind::Body => &mut segment_set.bodies,
                        };

                        kind.insert(point, segment);
                    }
                }
            }
        }

        // Determine the changes for each species
        for (species, segment_set) in snakes {
            changes.extend(self.determine_species_changes(&map, species, segment_set));
        }

        changes
    }

    fn determine_species_changes(
        &mut self,
        map: &Map,
        species: SnakeSpecies,
        mut segment_set: SnakeSegmentSet,
    ) -> Vec<Change> {
        let mut changes = Vec::new();

        // Determine where to move to each existing snake
        for (point, head) in segment_set.heads {
            let tail =
                self.extract_snake_tail(species, point, head.next_segment, &mut segment_set.bodies);

            match tail {
                None => {
                    // This snake is now invalid and dies
                    changes.push(Change::Die(point));
                }
                Some(tail) => {
                    // TODO
                }
            }
        }

        // Dangling bodies die
        for point in segment_set.bodies.into_keys() {
            changes.push(Change::Die(point));
        }

        // Detect new snakes
        let spare_parts = &mut segment_set.spare_parts;

        changes
    }

    fn apply_changes(&self, changes: Vec<Change>) {
        todo!()
    }

    /// Find a new snake that contains the given `point`. The snake orientation will be randomly
    /// chosen. Also, if there are multiple ambiguous snake formations, the result will be randomly
    /// determined.
    ///
    /// The returned segments are ordered such that the head is the first element. Note that the
    /// point passed as argument does not need to be the actual head nor tail.
    fn determine_new_snake(
        &mut self,
        map: &Map,
        point: Point,
        species: SnakeSpecies,
        visited: &mut HashSet<Point>,
    ) -> Option<Vec<Point>> {
        let max_size = self.max_size(species);
        let mut segments = VecDeque::with_capacity(max_size);

        let mut head = point;
        let mut tail = point;
        segments.push_front(point);

        let mut candidates = Vec::with_capacity(8);
        while segments.len() < max_size {
            candidates.clear();
            for (new_head, base) in [(true, head), (false, tail)] {
                for direction in [Point::X, Point::Y, -Point::X, -Point::Y] {
                    let candidate = base + direction;
                    let is_new_snake_segment = map
                        .cells()
                        .get(candidate)
                        .and_then(|cell| cell.animal().snake())
                        .map(|snake| snake.segment.is_none() && snake.species == species)
                        .unwrap_or(false);

                    if is_new_snake_segment && !visited.contains(&candidate) {
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
                    visited.insert(candidate);
                }
            }
        }

        (segments.len() >= self.min_size).then(|| segments.into())
    }

    /// Find the tail of the snake beginning at a given head. It will only return the tail if the
    /// snake is big enough.
    ///
    /// This will also remove all the referenced body segments from the set, so that they cannot be
    /// used as part of another snake.
    fn extract_snake_tail(
        &self,
        species: SnakeSpecies,
        head: Point,
        head_next_segment: Option<Point>,
        body_segment_set: &mut HashMap<Point, SnakeSegment>,
    ) -> Option<Point> {
        let max_size = self.max_size(species);
        let mut tail = head;
        let mut size = 1;
        let mut next_segment = head_next_segment;

        while let (Some(target), true) = (next_segment, size < max_size) {
            match body_segment_set.remove(&target) {
                None => break,
                Some(snake_segment) => {
                    tail = target;
                    size += 1;
                    next_segment = snake_segment.next_segment;
                }
            }
        }

        (size >= self.min_size).then_some(tail)
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
}
