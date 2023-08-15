use crate::cell::Cell;
use crate::cell_color::CellColor;
use crate::config::Config;
use crate::point::Point;
use anyhow::{ensure, Context, Result};
use image::{GenericImageView, Pixel};
use itertools::Itertools;
use ndarray::{s, Array2};
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
    pub fn new(config: &Config) -> Result<Self> {
        let size = config.map_size;
        let image = image::open(&config.height_map)?;

        ensure!(
            image.dimensions() == (size as u32, size as u32),
            "the height map must be {}x{}",
            size,
            size
        );
        let mut cells = Array2::from_shape_fn((size, size), |(i, j)| {
            let height = image.get_pixel(j as u32, i as u32).to_luma()[0];

            Cell::empty(height)
        });

        // Normalize heights to stretch the full range 0 to 255
        let (min_height, max_height) = cells
            .iter()
            .map(|cell| cell.height())
            .minmax()
            .into_option()
            .context("not empty map")?;
        let factor = 255.0 / (max_height - min_height) as f64;
        for cell in &mut cells {
            let normal_height = (cell.height() - min_height) as f64 * factor;
            cell.set_height(normal_height.round() as u8);
        }

        Ok(Map {
            version_id: Self::now(),
            cells,
            change_notifier: Default::default(),
        })
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

    pub fn set_cell_color(&mut self, point: Point, color: CellColor) -> Result<()> {
        let cell = self.cells.get_mut(point).context("invalid cell position")?;

        cell.with_color(color)?;
        self.notify_update();

        Ok(())
    }

    pub fn change_notifier(&self) -> &Arc<Notify> {
        &self.change_notifier
    }

    pub fn notify_update(&mut self) {
        self.version_id = Self::now();
        self.change_notifier.notify_waiters();
    }

    /// Return exclusive references to two distinct cells.
    ///
    /// # Panics
    /// If the points are the same or out of bounds
    pub fn two_cells_mut(&mut self, a: Point, b: Point) -> (&mut Cell, &mut Cell) {
        let (cell_a, cell_b) = self.cells.multi_slice_mut((
            s![a.y as usize, a.x as usize],
            s![b.y as usize, b.x as usize],
        ));

        (cell_a.into_scalar(), cell_b.into_scalar())
    }

    /// Return exclusive references to three distinct cells.
    ///
    /// # Panics
    /// If the points are the same or out of bounds
    pub fn three_cells_mut(
        &mut self,
        a: Point,
        b: Point,
        c: Point,
    ) -> (&mut Cell, &mut Cell, &mut Cell) {
        let (cell_a, cell_b, cell_c) = self.cells.multi_slice_mut((
            s![a.y as usize, a.x as usize],
            s![b.y as usize, b.x as usize],
            s![c.y as usize, c.x as usize],
        ));

        (
            cell_a.into_scalar(),
            cell_b.into_scalar(),
            cell_c.into_scalar(),
        )
    }

    fn now() -> String {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("must be after epoch")
            .as_nanos()
            .to_string()
    }
}
