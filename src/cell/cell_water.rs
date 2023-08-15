#[derive(Debug, Clone, Copy)]
pub enum CellWater {
    Empty,
    Shallow,
    Deep,
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
