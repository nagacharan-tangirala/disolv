use disolv_core::metrics::Metric;
use serde::Deserialize;
use std::ops::{Add, AddAssign};

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Energy(u64);

impl Energy {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Metric for Energy {}

impl Add for Energy {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

#[derive(Default, Deserialize, Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct MegaHertz(u64);

impl MegaHertz {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Metric for MegaHertz {}

impl Add for MegaHertz {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl AddAssign for MegaHertz {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
