use crate::message::Metadata;

/// A trait that measures some quantity of the radio. It could be a struct or a simple named type.
/// Any number of metrics can be used to measure the radio usage. The name should be unique and
/// must be added to the enum that implements the <code>MetricName</code> trait.
pub trait Metric: Default + PartialEq + PartialOrd + Copy + Clone + Send + Sync {
    fn as_f32(&self) -> f32;
}

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

/// A trait that can be used to contain the measurement process of a metric. It must be applied on
/// each struct that measure some kind of a metric.
pub trait Measurable<M>: Clone + Send + Sync
where
    M: Metric,
{
    type P: Metadata;
    fn measure(&mut self, metadata: &Self::P) -> Feasibility<M>;
}

/// A trait that can be used to assess whether sufficient radio resources are available
/// to transfer a payload. It must be applied on each struct that assesses the feasibility of a
/// payload transmission.
pub trait Consumable<M>: Clone + Send + Sync
where
    M: Metric,
{
    type P: Metadata;
    fn consume(&mut self, metadata: &Self::P) -> Feasibility<M>;
}
