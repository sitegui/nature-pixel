use crate::cell_color::CellColor;
use anyhow::{bail, Result};

#[derive(Debug, Clone)]
pub struct Cell {
    animal: CellAnimal,
    water: CellWater,
    grass: CellGrass,
    height: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum CellAnimal {
    Empty,
    Ant,
    Frog,
    Snake1,
    Snake2,
    Snake3,
    Dead,
}

#[derive(Debug, Clone, Copy)]
pub enum CellWater {
    Empty,
    Shallow,
    Deep,
}

#[derive(Debug, Clone, Copy)]
pub enum CellGrass {
    Empty,
    Dry,
    Low,
    High,
}

impl Cell {
    pub fn empty(height: u8) -> Self {
        Cell {
            animal: CellAnimal::Empty,
            water: CellWater::Empty,
            grass: CellGrass::Empty,
            height,
        }
    }

    pub fn color(&self) -> CellColor {
        match self.animal {
            CellAnimal::Ant => CellColor::Ant,
            CellAnimal::Frog => CellColor::Frog,
            CellAnimal::Snake1 => CellColor::Snake1,
            CellAnimal::Snake2 => CellColor::Snake2,
            CellAnimal::Snake3 => CellColor::Snake3,
            CellAnimal::Dead => CellColor::DeadMatter,
            CellAnimal::Empty => match self.water {
                CellWater::Shallow => CellColor::ShallowWater,
                CellWater::Deep => CellColor::DeepWater,
                CellWater::Empty => match self.grass {
                    CellGrass::Dry => CellColor::DryGrass,
                    CellGrass::Low => CellColor::LowGrass,
                    CellGrass::High => CellColor::HighGrass,
                    CellGrass::Empty => CellColor::Empty,
                },
            },
        }
    }

    pub fn with_color(&mut self, color: CellColor) -> Result<()> {
        match color {
            CellColor::Empty => {
                self.animal = CellAnimal::Empty;
                self.water = CellWater::Empty;
                self.grass = CellGrass::Empty;
            }
            CellColor::Ant => self.animal = CellAnimal::Ant,
            CellColor::Frog => self.animal = CellAnimal::Frog,
            CellColor::Snake1 => self.animal = CellAnimal::Snake1,
            CellColor::Snake2 => self.animal = CellAnimal::Snake2,
            CellColor::Snake3 => self.animal = CellAnimal::Snake3,
            CellColor::ShallowWater => self.water = CellWater::Shallow,
            CellColor::LowGrass => self.grass = CellGrass::Low,
            _ => {
                bail!("cannot set such color")
            }
        }

        Ok(())
    }

    pub fn animal(&self) -> CellAnimal {
        self.animal
    }
    pub fn water(&self) -> CellWater {
        self.water
    }
    pub fn grass(&self) -> CellGrass {
        self.grass
    }
    pub fn set_animal(&mut self, animal: CellAnimal) {
        self.animal = animal;
    }
    pub fn set_water(&mut self, water: CellWater) {
        self.water = water;
    }
    pub fn set_grass(&mut self, grass: CellGrass) {
        self.grass = grass;
    }
    pub fn height(&self) -> u8 {
        self.height
    }
    pub fn set_height(&mut self, height: u8) {
        self.height = height;
    }
}

impl CellWater {
    pub fn drier(self) -> Option<Self> {
        match self {
            CellWater::Empty => None,
            CellWater::Shallow => Some(CellWater::Empty),
            CellWater::Deep => Some(CellWater::Shallow),
        }
    }

    pub fn wetter(self) -> Option<Self> {
        match self {
            CellWater::Empty => Some(CellWater::Shallow),
            CellWater::Shallow => Some(CellWater::Deep),
            CellWater::Deep => None,
        }
    }
}
