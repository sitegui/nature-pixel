use crate::cell_color::CellColor;
use crate::ecosystem::snake::{Snake, SnakeSpecies};
use anyhow::{bail, Result};
use cell_animal::CellAnimal;
use cell_grass::CellGrass;
use cell_water::CellWater;

pub mod cell_animal;
pub mod cell_grass;
pub mod cell_water;

#[derive(Debug)]
pub struct Cell {
    animal: CellAnimal,
    water: CellWater,
    grass: CellGrass,
    height: u8,
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
        match &self.animal {
            CellAnimal::Insect(_) => CellColor::Insect,
            CellAnimal::Amphibian(_) => CellColor::Amphibian,
            CellAnimal::Snake(snake) => match snake.species() {
                SnakeSpecies::A => CellColor::SnakeA,
                SnakeSpecies::B => CellColor::SnakeB,
                SnakeSpecies::C => CellColor::SnakeC,
            },
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
            CellColor::Insect => self.animal = CellAnimal::Insect(Default::default()),
            CellColor::Amphibian => self.animal = CellAnimal::Amphibian(Default::default()),
            CellColor::SnakeA => {
                self.animal = CellAnimal::Snake(Box::new(Snake::new(SnakeSpecies::A)))
            }
            CellColor::SnakeB => {
                self.animal = CellAnimal::Snake(Box::new(Snake::new(SnakeSpecies::B)))
            }
            CellColor::SnakeC => {
                self.animal = CellAnimal::Snake(Box::new(Snake::new(SnakeSpecies::C)))
            }
            CellColor::ShallowWater => self.water = CellWater::Shallow,
            CellColor::LowGrass => self.grass = CellGrass::Low,
            _ => {
                bail!("cannot set such color")
            }
        }

        Ok(())
    }

    pub fn animal(&self) -> &CellAnimal {
        &self.animal
    }
    pub fn animal_mut(&mut self) -> &mut CellAnimal {
        &mut self.animal
    }
    pub fn water(&self) -> CellWater {
        self.water
    }
    pub fn grass(&self) -> CellGrass {
        self.grass
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
