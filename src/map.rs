use crate::cell::Cell;
use ndarray::Array2;

#[derive(Debug)]
pub struct Map {
    cells: Array2<Cell>,
}

impl Map {
    pub fn new(size: usize) -> Self {
        Map {
            cells: Array2::from_elem((size, size), Cell::Empty),
        }
    }

    pub fn size(&self) -> usize {
        self.cells.nrows()
    }

    pub fn cells(&self) -> &Array2<Cell> {
        &self.cells
    }
}
