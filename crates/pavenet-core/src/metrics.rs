use pavenet_engine::metrics::Metric;
use serde::Deserialize;
use std::fmt::Display;
use std::ops::{Add, AddAssign};

#[derive(Debug, Clone, Copy)]
pub enum RadioMetrics {
    Latency,
    Throughput,
    PacketLoss,
}

#[derive(Deserialize, Debug, Clone, PartialEq, PartialOrd, Default, Copy)]
pub struct Latency(f32);

impl Display for Latency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}ms", self.0)
    }
}

impl Latency {
    pub fn new(value: f32) -> Self {
        Self(value)
    }
    pub fn as_u32(&self) -> u32 {
        self.0 as u32
    }
}

impl From<f32> for Latency {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl Metric for Latency {
    fn as_f32(&self) -> f32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Copy)]
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

impl Metric for Throughput {
    fn as_f32(&self) -> f32 {
        self.0 as f32
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Copy)]
pub struct Bandwidth(u32);

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

impl SubAssign for Bandwidth {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl Metric for Bandwidth {}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, PartialOrd, Default, Copy)]
pub struct Bytes(u64);

impl Display for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}B", self.0)
    }
}

impl Bytes {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl SubAssign for Bytes {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl AddAssign for Bytes {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl Add for Bytes {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl Sub for Bytes {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0)
    }
}

impl Sum for Bytes {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), |acc, x| acc + x)
    }
}

impl Metric for Bytes {}
