use crate::cell::Cell;
use crate::cell_color::CellColor;
use anyhow::{Context, Result};
use ndarray::Array2;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct Map {
    version_id: u64,
    cells: Array2<Cell>,
}

impl Map {
    pub fn new(size: usize) -> Self {
        Map {
            version_id: Self::now(),
            cells: Array2::from_elem((size, size), Cell::Empty),
        }
    }

    pub fn size(&self) -> usize {
        self.cells.nrows()
    }

    pub fn cells(&self) -> &Array2<Cell> {
        &self.cells
    }

    pub fn version_id(&self) -> u64 {
        self.version_id
    }

    pub fn set_cell_color(&mut self, x: usize, y: usize, color: CellColor) -> Result<()> {
        let cell = self
            .cells
            .get_mut([y, x])
            .context("invalid cell position")?;

        cell.with_color(color)?;

        Ok(())
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("must be after epoch")
            .as_secs()
    }
}
