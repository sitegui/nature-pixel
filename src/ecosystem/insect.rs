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
pub struct Insect(SimpleAnimal);

#[derive(Debug)]
pub struct InsectSystem(SimpleAnimalSystem<Insect>);

impl InsectSystem {
    pub fn new(config: &Config, map: Arc<RwLock<Map>>) -> Self {
        Self(SimpleAnimalSystem::new(
            Duration::from_secs(config.insect_tick_seconds),
            config.insect_eating_radius,
            config.insect_mating_radius,
            config.insect_destination_radius,
            Duration::from_secs(config.insect_starvation_delay_seconds),
            map,
        ))
    }

    pub async fn run(self) {
        self.0.run().await
    }
}

impl SimpleAnimalKind for Insect {
    type WalkCandidates = array::IntoIter<WalkCandidate, 2>;

    fn get(cell: &Cell) -> Option<&SimpleAnimal> {
        cell.animal().insect().map(|insect| &insect.0)
    }

    fn get_mut(cell: &mut Cell) -> Option<&mut SimpleAnimal> {
        cell.animal_mut().insect_mut().map(|insect| &mut insect.0)
    }

    fn walk_candidates(point: Point, direction: Point) -> Self::WalkCandidates {
        [
            WalkCandidate::new(point, direction.turn_right(), 1),
            WalkCandidate::new(point, direction.turn_left(), 1),
        ]
        .into_iter()
    }

    fn is_food_goal(cell: &Cell) -> bool {
        cell.animal().is_dead()
    }

    fn is_mating_ground_goal(cell: &Cell) -> bool {
        !cell.grass().is_empty()
    }

    fn build_cell(simple_animal: SimpleAnimal) -> CellAnimal {
        CellAnimal::Insect(Box::new(Insect(simple_animal)))
    }
}
