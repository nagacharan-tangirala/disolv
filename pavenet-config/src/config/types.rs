use serde_derive::Deserialize;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct DeviceId(u32);

impl Display for DeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:09}", self.0)
    }
}

impl FromStr for DeviceId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse::<u32>()?;
        Ok(Self(id))
    }
}

impl Into<u32> for DeviceId {
    fn into(self) -> u32 {
        self.0
    }
}

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimeStamp(pub u64);

impl Display for TimeStamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:09}", self.0)
    }
}

impl FromStr for TimeStamp {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse::<u64>()?;
        Ok(Self(id))
    }
}

impl From<u64> for TimeStamp {
    fn from(f: u64) -> Self {
        Self(f)
    }
}

impl Into<u64> for TimeStamp {
    fn into(self) -> u64 {
        self.0
    }
}

impl TimeStamp {
    pub fn as_f32(&self) -> f32 {
        self.0 as f32
    }
}

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
