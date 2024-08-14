use std::fmt::{Debug, Display};
use std::ops::{Add, AddAssign, Div, Mul};
use std::str::FromStr;

use serde::Deserialize;

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimeMS(pub u64);

impl Display for TimeMS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TimeMS {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse::<u64>()?;
        Ok(Self(id))
    }
}

impl From<u64> for TimeMS {
    fn from(f: u64) -> Self {
        Self(f)
    }
}

impl From<i32> for TimeMS {
    fn from(f: i32) -> Self {
        Self(f as u64)
    }
}

impl From<i64> for TimeMS {
    fn from(f: i64) -> Self {
        Self(f as u64)
    }
}

impl TimeMS {
    pub fn as_u64(&self) -> u64 {
        self.0
    }
    pub fn as_i64(&self) -> i64 {
        self.0 as i64
    }
    pub fn as_f32(&self) -> f32 {
        self.0 as f32
    }
}

impl Mul for TimeMS {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Div for TimeMS {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Add for TimeMS {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for TimeMS {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

/// A trait passed to the agent so that an agent can access other agents. Any common models
/// applicable to all the agents irrespective of type should be assigned to a struct that
/// implements this trait.
pub trait Bucket: Send {
    fn initialize(&mut self, step: TimeMS);
    fn before_agents(&mut self, step: TimeMS);
    fn after_stage_one(&mut self) {}
    fn after_stage_two(&mut self) {}
    fn after_stage_three(&mut self) {}
    fn after_stage_four(&mut self) {}
    fn after_agents(&mut self);
    fn stream_input(&mut self);
    fn stream_output(&mut self);
    fn terminate(self);
}

#[cfg(test)]
pub mod tests {}
