use crate::cell::cell_animal::CellAnimal;
use crate::config::Config;
use crate::map::Map;
use crate::monitored_rwlock::MonitoredRwLock;
use crate::point::Point;
use rand::prelude::{Distribution, SliceRandom, SmallRng};
use rand::SeedableRng;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

#[derive(Debug)]
pub struct Snake {
    species: SnakeSpecies,
    segment: Option<SnakeSegment>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SnakeSpecies {
    A,
    B,
    C,
}

#[derive(Debug, Clone, Copy)]
struct SnakeSegment {
    kind: SnakeSegmentKind,
    /// The direction to follow in order to get to the next segment. `None` if this is a tail.
    next_segment: Option<Point>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
        let map = self.map.read(module_path!());
        let mut visited = HashSet::new();
        let mut maybe_dangling_bodies = HashSet::new();

        for (ij, cell) in map.cells().indexed_iter() {
            if let CellAnimal::Snake(snake) = cell.animal() {
                let point = Point::new_ij(ij);
                if !visited.insert(point) {
                    // This cell was already handled this tick
                    continue;
                }

                let change = match snake.segment {
                    None => self
                        .determine_new_snake(&map, point, snake.species, &mut visited)
                        .map(Change::NewSnake),
                    Some(segment) => match segment.kind {
                        SnakeSegmentKind::Head => self.determine_movement(),
                        SnakeSegmentKind::Body => {
                            maybe_dangling_bodies.insert(point);
                        }
                    },
                };

                if let Some(change) = change {
                    changes.push(change);
                }
            }
        }

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
    fn find_snake_tail(
        &self,
        map: &Map,
        head: Point,
        species: SnakeSpecies,
        head_next_segment: Option<Point>,
        visited: &mut HashSet<Point>,
    ) -> Option<Point> {
        let max_size = self.max_size(species);
        let mut tail = head;
        let mut size = 1;
        let mut next_segment = head_next_segment;

        while let (Some(delta), true) = (next_segment, size < max_size) {
            let target = tail + delta;

            let snake_segment = map
                .cells()
                .get(target)
                .and_then(|cell| cell.animal().snake())
                .filter(|snake| snake.species == species)
                .and_then(|snake| snake.segment)
                .filter(|segment| segment.kind == SnakeSegmentKind::Body);

            match snake_segment {
                None => break,
                Some(snake_segment) => {
                    // Mark this other cell as visited
                    if !visited.insert(target) {
                        break;
                    }

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
