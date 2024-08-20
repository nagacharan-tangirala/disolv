use std::fmt::Display;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use serde::{Deserialize, Serialize};

use crate::message::Metadata;

/// A trait that measures some quantity of the radio. It could be a struct or a simple named type.
/// Any number of metrics can be used to measure the radio usage. The name should be unique and
/// must be added to the enum that implements the <code>MetricName</code> trait.
pub trait Metric: Default + PartialEq + PartialOrd + Copy + Clone + Send + Sync {}

/// Each byte is 8 bits, a custom type is defined.
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

    pub fn as_f64(&self) -> f64 {
        self.0 as f64
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

/// An enum that represents the feasibility of a metric. This is used as return type of the
/// feasibility evaluation so that the caller can get the feasibility and the actual value of the
/// metric that was measured.
#[derive(Clone, Copy, Debug)]
pub enum Feasibility<M>
where
    M: Metric,
{
    Feasible(M),
    Infeasible(M),
}

/// A trait that can be used on structs that contain the settings of a metric. The settings should
/// be capable of building an instance of the struct that implements either the
/// <code>Measurable</code> or <code>Consumable</code> trait.
pub trait MetricSettings {}

/// A trait that can be used to contain the measurement process of a metric. It must be applied on
/// each struct that measures some kind of radio metric.
pub trait Measurable<M, P>: Clone + Send + Sync
where
    M: Metric,
    P: Metadata,
{
    type S: MetricSettings;
    fn with_settings(settings: &Self::S) -> Self;
    fn measure(&mut self, metadata: &P) -> Feasibility<M>;
}

/// A trait that can be used to define a consumable that can be reset at each time step.
/// This can be used to define resources that are measured as they are consumed.
pub trait Consumable<M, P>: Clone + Send + Sync
where
    M: Metric,
    P: Metadata,
{
    type S: MetricSettings;
    fn with_settings(settings: Self::S) -> Self;
    fn reset(&mut self);
    fn consume(&mut self, metadata: &P) -> Feasibility<M>;
    fn available(&self) -> M;
}

/// A trait that can be used to define a resource that cannot be reset.
/// Trying to consume more than the available resource should be infeasible.
pub trait Resource<M, P>: Clone + Send + Sync
where
    M: Metric,
    P: Metadata,
{
    type S: MetricSettings;
    fn with_settings(settings: &Self::S) -> Self;
    fn consume(&mut self, metadata: &P) -> Feasibility<M>;
    fn available(&self) -> M;
}
