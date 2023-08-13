#[derive(Debug, Clone, Copy)]
pub enum CellColor {
    White,
    LightGreen,
    DarkGreen,
}

impl CellColor {
    pub const CSS_STRINGS: &'static [&'static str] = &["white", "limegreen", "darkgreen"];

    pub fn as_index(self) -> usize {
        self as usize
    }
}
