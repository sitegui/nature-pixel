use crate::cell_color::CellColor;
use anyhow::{bail, Result};

#[derive(Debug, Clone)]
pub enum Cell {
    Empty,
    Ant,
    Frog,
    Snake1,
    Snake2,
    Snake3,
    DryGrass,
    LowGrass,
    HighGrass,
    ShallowWater,
    DeepWater,
    DeadMatter,
}

impl Cell {
    pub fn color(&self) -> CellColor {
        match self {
            Cell::Empty => CellColor::Empty,
            Cell::Ant => CellColor::Ant,
            Cell::Frog => CellColor::Frog,
            Cell::Snake1 => CellColor::Snake1,
            Cell::Snake2 => CellColor::Snake2,
            Cell::Snake3 => CellColor::Snake3,
            Cell::DryGrass => CellColor::DryGrass,
            Cell::LowGrass => CellColor::LowGrass,
            Cell::HighGrass => CellColor::HighGrass,
            Cell::ShallowWater => CellColor::ShallowWater,
            Cell::DeepWater => CellColor::DeepWater,
            Cell::DeadMatter => CellColor::DeadMatter,
        }
    }

    pub fn with_color(&mut self, color: CellColor) -> Result<()> {
        match color {
            CellColor::Empty => *self = Cell::Empty,
            CellColor::Ant => *self = Cell::Ant,
            CellColor::Frog => *self = Cell::Frog,
            CellColor::Snake1 => *self = Cell::Snake1,
            CellColor::Snake2 => *self = Cell::Snake2,
            CellColor::Snake3 => *self = Cell::Snake3,
            CellColor::LowGrass => *self = Cell::LowGrass,
            CellColor::ShallowWater => *self = Cell::ShallowWater,
            _ => {
                bail!("cannot set such color")
            }
        }

        Ok(())
    }
}
