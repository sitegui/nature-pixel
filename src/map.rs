use crate::cell::Cell;
use crate::cell_color::CellColor;
use anyhow::{Context, Result};
use ndarray::Array2;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Notify;

#[derive(Debug)]
pub struct Map {
    version_id: String,
    cells: Array2<Cell>,
    change_notifier: Arc<Notify>,
}

impl Map {
    pub fn new(size: usize) -> Self {
        Map {
            version_id: Self::now(),
            cells: Array2::from_elem((size, size), Cell::empty()),
            change_notifier: Default::default(),
        }
    }

    pub fn size(&self) -> usize {
        self.cells.nrows()
    }

    pub fn cells(&self) -> &Array2<Cell> {
        &self.cells
    }

    pub fn cells_mut(&mut self) -> &mut Array2<Cell> {
        &mut self.cells
    }

    pub fn version_id(&self) -> &str {
        &self.version_id
    }

    pub fn set_cell_color(&mut self, x: usize, y: usize, color: CellColor) -> Result<()> {
        let cell = self
            .cells
            .get_mut([y, x])
            .context("invalid cell position")?;

        cell.with_color(color)?;
        self.notify_update();

        Ok(())
    }

    pub fn change_notifier(&self) -> &Arc<Notify> {
        &self.change_notifier
    }

    fn now() -> String {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("must be after epoch")
            .as_nanos()
            .to_string()
    }

    fn notify_update(&mut self) {
        self.version_id = Self::now();
        self.change_notifier.notify_waiters();
    }
}
