use crate::message::{Metadata, TxReport};

/// A trait that measures some quantity of the radio. It could be a struct or a simple named type.
/// Any number of metrics can be used to measure the radio usage. The name should be unique and
/// must be added to the enum that implements the <code>MetricName</code> trait.
pub trait Metric: Default + PartialEq + PartialOrd + Copy + Clone + Send + Sync {}

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
