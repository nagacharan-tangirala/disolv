use pavenet_engine::bucket::TimeStamp;
use serde::Deserialize;
use std::fmt::Display;
use std::ops::{Add, AddAssign, Mul};
use std::str::FromStr;

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimeS(pub u64);

impl Display for TimeS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:09}", self.0)
    }
}

impl FromStr for TimeS {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse::<u64>()?;
        Ok(Self(id))
    }
}

impl From<u64> for TimeS {
    fn from(f: u64) -> Self {
        Self(f)
    }
}

impl From<i64> for TimeS {
    fn from(f: i64) -> Self {
        Self(f as u64)
    }
}

impl TimeS {
    pub fn as_u64(&self) -> u64 {
        self.0
    }
    pub fn as_i64(&self) -> i64 {
        self.0 as i64
    }
}

impl Mul for TimeS {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Add for TimeS {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl AddAssign for TimeS {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl TimeStamp for TimeS {
    fn as_f32(&self) -> f32 {
        self.0 as f32
    }
}
