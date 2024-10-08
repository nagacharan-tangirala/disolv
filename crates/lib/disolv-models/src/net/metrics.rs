use std::fmt::Display;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use serde::Deserialize;
use serde::Serialize;

use disolv_core::metrics::Metric;

#[derive(Debug, Clone, Copy)]
pub enum RadioMetricTypes {
    Latency,
    Bandwidth,
    Throughput,
    PacketLoss,
}

#[derive(Deserialize, Debug, Clone, PartialEq, PartialOrd, Default, Copy)]
pub struct Latency(u64);

impl Display for Latency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}ms", self.0)
    }
}

impl Latency {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Sub for Latency {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0)
    }
}

impl Metric for Latency {}

#[derive(Deserialize, Debug, Clone, PartialEq, PartialOrd, Default, Copy)]
pub struct Throughput(u32);

impl AddAssign for Throughput {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl Add for Throughput {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl Metric for Throughput {}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, PartialOrd, Default, Copy)]
pub struct Bandwidth(u64);

impl Bandwidth {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl AddAssign for Bandwidth {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl Add for Bandwidth {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl Sub for Bandwidth {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Bandwidth {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl Metric for Bandwidth {}
