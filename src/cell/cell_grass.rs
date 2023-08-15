#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CellGrass {
    Empty,
    Dry,
    Low,
    High,
}

impl CellGrass {
    pub fn is_empty(self) -> bool {
        self == CellGrass::Empty
    }
}