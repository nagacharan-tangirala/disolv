use serde_derive::Deserialize;
use std::fmt::Display;
use std::ops::Add;
use std::str::FromStr;

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimeStamp(pub u64);

impl Add for TimeStamp {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

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

impl From<i64> for TimeStamp {
    fn from(f: i64) -> Self {
        Self(f as u64)
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
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}
