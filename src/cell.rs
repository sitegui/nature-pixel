use crate::cell_color::CellColor;
use anyhow::{bail, Result};

#[derive(Debug, Clone)]
pub enum Cell {
    Empty,
    LowGrass,
    HighGrass,
}

impl Cell {
    pub fn color(&self) -> CellColor {
        match self {
            Cell::Empty => CellColor::White,
            Cell::LowGrass => CellColor::LightGreen,
            Cell::HighGrass => CellColor::DarkGreen,
        }
    }

    pub fn with_color(&mut self, color: CellColor) -> Result<()> {
        match color {
            CellColor::White => *self = Cell::Empty,
            CellColor::LightGreen => *self = Cell::LowGrass,
            CellColor::DarkGreen => {
                bail!("cannot set such color")
            }
        }

        Ok(())
    }
}
