use crate::cell_color::CellColor;

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
}
