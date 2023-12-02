use pavenet_engine::metrics::Metric;
use serde::Deserialize;
use std::fmt::Display;
use std::ops::{Add, AddAssign, Mul, Sub};

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

impl Metric for Bandwidth {
    fn as_f32(&self) -> f32 {
        self.0 as f32
    }
}
