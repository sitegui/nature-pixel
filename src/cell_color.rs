use anyhow::bail;
use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub enum CellColor {
    Empty,
    Insect,
    Amphibian,
    SnakeA,
    SnakeB,
    SnakeC,
    DryGrass,
    LowGrass,
    HighGrass,
    ShallowWater,
    DeepWater,
    DeadMatter,
}

impl CellColor {
    /// All known colors, sorted by their index
    pub const ALL_COLORS: &'static [CellColor] = &[
        CellColor::Empty,
        CellColor::Insect,
        CellColor::Amphibian,
        CellColor::SnakeA,
        CellColor::SnakeB,
        CellColor::SnakeC,
        CellColor::DryGrass,
        CellColor::LowGrass,
        CellColor::HighGrass,
        CellColor::ShallowWater,
        CellColor::DeepWater,
        CellColor::DeadMatter,
    ];

    /// Colors that are available for user interaction
    pub const AVAILABLE_COLORS: &'static [CellColor] = &[
        CellColor::Empty,
        CellColor::Insect,
        CellColor::Amphibian,
        CellColor::SnakeA,
        CellColor::SnakeB,
        CellColor::SnakeC,
        CellColor::LowGrass,
        CellColor::ShallowWater,
    ];

    pub fn as_index(self) -> usize {
        self as usize
    }

    pub fn as_rgb(self) -> [u8; 3] {
        match self {
            CellColor::Empty => [255, 255, 255],
            CellColor::Insect => [50, 18, 16],
            CellColor::Amphibian => [188, 226, 61],
            CellColor::SnakeA => [229, 205, 23],
            CellColor::SnakeB => [217, 158, 47],
            CellColor::SnakeC => [184, 83, 55],
            CellColor::DryGrass => [171, 144, 101],
            CellColor::LowGrass => [99, 130, 86],
            CellColor::HighGrass => [27, 116, 72],
            CellColor::ShallowWater => [47, 168, 232],
            CellColor::DeepWater => [9, 70, 99],
            CellColor::DeadMatter => [123, 123, 123],
        }
    }

    pub fn try_from_index(index: usize) -> Result<Self> {
        match index {
            0 => Ok(CellColor::Empty),
            1 => Ok(CellColor::Insect),
            2 => Ok(CellColor::Amphibian),
            3 => Ok(CellColor::SnakeA),
            4 => Ok(CellColor::SnakeB),
            5 => Ok(CellColor::SnakeC),
            6 => Ok(CellColor::DryGrass),
            7 => Ok(CellColor::LowGrass),
            8 => Ok(CellColor::HighGrass),
            9 => Ok(CellColor::ShallowWater),
            10 => Ok(CellColor::DeepWater),
            11 => Ok(CellColor::DeadMatter),
            _ => bail!("invalid color index: {}", index),
        }
    }
}
