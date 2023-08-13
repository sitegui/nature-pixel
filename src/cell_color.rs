use anyhow::{bail, Error};
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub enum CellColor {
    Empty,
    Ant,
    Frog,
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
        CellColor::Ant,
        CellColor::Frog,
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
        CellColor::Ant,
        CellColor::Frog,
        CellColor::Snake1,
        CellColor::Snake2,
        CellColor::Snake3,
        CellColor::LowGrass,
        CellColor::ShallowWater,
    ];

    pub fn as_index(self) -> usize {
        self as usize
    }

    pub fn as_str(self) -> &'static str {
        match self {
            CellColor::Empty => "#ffffff",
            CellColor::Ant => "#321210",
            CellColor::Frog => "#bce23d",
            CellColor::Snake1 => "#e5cd17",
            CellColor::Snake2 => "#d99e2f",
            CellColor::Snake3 => "#b85337",
            CellColor::DryGrass => "#ab9065",
            CellColor::LowGrass => "#638256",
            CellColor::HighGrass => "#1b7448",
            CellColor::ShallowWater => "#2fa8e8",
            CellColor::DeepWater => "#094663",
            CellColor::DeadMatter => "#555555",
        }
    }
}

impl FromStr for CellColor {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "#ffffff" => Ok(CellColor::Empty),
            "#321210" => Ok(CellColor::Ant),
            "#bce23d" => Ok(CellColor::Frog),
            "#e5cd17" => Ok(CellColor::Snake1),
            "#d99e2f" => Ok(CellColor::Snake2),
            "#b85337" => Ok(CellColor::Snake3),
            "#ab9065" => Ok(CellColor::DryGrass),
            "#638256" => Ok(CellColor::LowGrass),
            "#1b7448" => Ok(CellColor::HighGrass),
            "#2fa8e8" => Ok(CellColor::ShallowWater),
            "#094663" => Ok(CellColor::DeepWater),
            "#555555" => Ok(CellColor::DeadMatter),
            _ => bail!("invalid color name: {}", s),
        }
    }
}

impl<'de> Deserialize<'de> for CellColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = String::deserialize(deserializer)?;
        name.parse().map_err(serde::de::Error::custom)
    }
}
