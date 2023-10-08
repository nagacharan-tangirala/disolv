use std::fmt::Display;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CellId(u32);

impl Display for CellId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for CellId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse::<u32>()?;
        Ok(Self(id))
    }
}

impl From<f32> for CellId {
    fn from(f: f32) -> Self {
        Self(f as u32)
    }
}

impl Into<u32> for CellId {
    fn into(self) -> u32 {
        self.0
    }
}
