use crate::cell::cell_animal::CellAnimal;
use crate::cell::Cell;
use crate::config::Config;
use crate::ecosystem::simple_animal::{
    SimpleAnimal, SimpleAnimalKind, SimpleAnimalSystem, WalkCandidate,
};
use crate::map::Map;
use crate::point::Point;
use std::array;
use std::sync::{Arc, RwLock};
use std::time::Duration;

#[derive(Debug, Default)]
pub struct Amphibian(SimpleAnimal);

#[derive(Debug)]
pub struct AmphibianSystem(SimpleAnimalSystem<Amphibian>);

impl AmphibianSystem {
    pub fn new(config: &Config, map: Arc<RwLock<Map>>) -> Self {
        Self(SimpleAnimalSystem::new(
            Duration::from_secs(config.amphibian_tick_seconds),
            config.amphibian_eating_radius,
            config.amphibian_mating_radius,
            config.amphibian_destination_radius,
            map,
        ))
    }

    pub async fn run(self) {
        self.0.run().await
    }
}

impl SimpleAnimalKind for Amphibian {
    type WalkCandidates = array::IntoIter<WalkCandidate, 4>;

    fn get(cell: &Cell) -> Option<&SimpleAnimal> {
        match cell.animal() {
            CellAnimal::Amphibian(amphibian) => Some(&amphibian.0),
            _ => None,
        }
    }

    fn get_mut(cell: &mut Cell) -> Option<&mut SimpleAnimal> {
        match cell.animal_mut() {
            CellAnimal::Amphibian(amphibian) => Some(&mut amphibian.0),
            _ => None,
        }
    }

    fn walk_candidates(point: Point, direction: Point) -> Self::WalkCandidates {
        [
            WalkCandidate::new(point, direction.turn_right(), 1),
            WalkCandidate::new(point, direction.turn_left(), 1),
            WalkCandidate::new(point, direction, 1),
            WalkCandidate::new(point, direction, 2),
        ]
        .into_iter()
    }

    fn is_food_goal(cell: &Cell) -> bool {
        cell.animal().insect().is_some()
    }

    fn is_mating_ground_goal(cell: &Cell) -> bool {
        !cell.water().is_empty()
    }

    fn build_cell(simple_animal: SimpleAnimal) -> CellAnimal {
        CellAnimal::Amphibian(Box::new(Amphibian(simple_animal)))
    }
}
