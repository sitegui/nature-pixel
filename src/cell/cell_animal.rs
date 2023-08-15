use crate::ecosystem::amphibian::Amphibian;
use crate::ecosystem::insect::Insect;

#[derive(Debug)]
pub enum CellAnimal {
    Empty,
    Insect(Box<Insect>),
    Amphibian(Box<Amphibian>),
    Snake1,
    Snake2,
    Snake3,
    Dead,
}

impl CellAnimal {
    pub fn is_empty(&self) -> bool {
        matches!(self, &CellAnimal::Empty)
    }
    pub fn is_dead(&self) -> bool {
        matches!(self, &CellAnimal::Dead)
    }
    pub fn insect(&self) -> Option<&Insect> {
        if let CellAnimal::Insect(insect) = self {
            Some(insect)
        } else {
            None
        }
    }
    pub fn insect_mut(&mut self) -> Option<&mut Insect> {
        if let CellAnimal::Insect(insect) = self {
            Some(insect)
        } else {
            None
        }
    }
}
