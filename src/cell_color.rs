use anyhow::{bail, Error};
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

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

impl FromStr for CellColor {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "white" => Ok(CellColor::White),
            "limegreen" => Ok(CellColor::LightGreen),
            "darkgreen" => Ok(CellColor::DarkGreen),
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
