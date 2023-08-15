use anyhow::bail;
use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub enum CellColor {
    Empty,
    Insect,
    Amphibian,
    Snake1,
    Snake2,
    Snake3,
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
        CellColor::Snake1,
        CellColor::Snake2,
        CellColor::Snake3,
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
        CellColor::Snake1,
        CellColor::Snake2,
        CellColor::Snake3,
        CellColor::LowGrass,
        CellColor::ShallowWater,
        CellColor::DeadMatter,
    ];

    pub fn as_index(self) -> usize {
        self as usize
    }

    pub fn as_rgb(self) -> [u8; 3] {
        match self {
            CellColor::Empty => [255, 255, 255],
            CellColor::Insect => [50, 18, 16],
            CellColor::Amphibian => [188, 226, 61],
            CellColor::Snake1 => [229, 205, 23],
            CellColor::Snake2 => [217, 158, 47],
            CellColor::Snake3 => [184, 83, 55],
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
            3 => Ok(CellColor::Snake1),
            4 => Ok(CellColor::Snake2),
            5 => Ok(CellColor::Snake3),
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
