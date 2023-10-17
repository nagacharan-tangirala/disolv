use pavenet_core::tier::Tier;
use pavenet_engine::bucket::TimeStamp;
use pavenet_engine::entity::Identifier;
use std::fmt::Display;
use std::ops::{Add, AddAssign};

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Nid(u32);

impl Identifier for Nid {}

impl Display for Nid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i32> for Nid {
    fn from(value: i32) -> Self {
        Self(value as u32)
    }
}

impl Into<u32> for Nid {
    fn into(self) -> u32 {
        self.0
    }
}

#[derive(Default, Clone, Copy, Debug, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct Ts(u32);

impl AddAssign for Ts {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Add for Ts {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl From<u64> for Ts {
    fn from(value: u64) -> Self {
        Self(value as u32)
    }
}

impl Into<f32> for Ts {
    fn into(self) -> f32 {
        self.0 as f32
    }
}

impl Into<u64> for Ts {
    fn into(self) -> u64 {
        self.0 as u64
    }
}

impl TimeStamp for Ts {}

impl Display for Ts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Default, Clone, Copy, Debug, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct Order(u32);

impl From<u32> for Order {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Into<i32> for Order {
    fn into(self) -> i32 {
        self.0 as i32
    }
}

impl Tier for Order {}
