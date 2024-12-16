use std::ops::{Add, AddAssign, Sub};

use rand_distr::num_traits::ToPrimitive;
use serde::Deserialize;

use disolv_core::metrics::Metric;

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
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
    pub fn as_f64(&self) -> f64 {
        self.0.to_f64().expect("failed to convert cpu to f64")
    }
}

impl Metric for MegaHertz {}

impl Add for MegaHertz {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl Sub for MegaHertz {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl AddAssign for MegaHertz {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
