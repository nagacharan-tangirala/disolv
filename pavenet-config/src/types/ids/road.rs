use std::fmt::Display;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RoadId(u32);

impl Display for RoadId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for RoadId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse::<u32>()?;
        Ok(Self(id))
    }
}

impl From<f32> for RoadId {
    fn from(f: f32) -> Self {
        Self(f as u32)
    }
}

impl Into<u32> for RoadId {
    fn into(self) -> u32 {
        self.0
    }
}
