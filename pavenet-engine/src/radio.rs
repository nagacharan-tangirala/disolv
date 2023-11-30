use crate::message::Queryable;
use crate::message::{GPayload, Metadata, NodeState};
use crate::node::NodeId;
use std::fmt::Debug;
use std::ops::{Add, AddAssign};
use typed_builder::TypedBuilder;

/// A trait that measures some quantity of the radio. It could be a struct or a simple named type.
/// Any number of metrics can be used to measure the radio usage. The name should be unique and
/// must be added to the enum that implements the <code>MetricName</code> trait.
pub trait Metric:
    Default + AddAssign + Add<Output = Self> + PartialEq + PartialOrd + Copy + Clone + Send + Sync
{
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

/// Use this trait to mark a type as a rule. This trait is used to enforce rules
/// on the payload transmission. An enum can be used to implement this trait with
/// each variant representing a rule.
pub trait TxRules: Default + Copy + Clone + Send + Sync {}

#[derive(Clone, Debug, TypedBuilder)]
pub struct RoutingRules<Q, T>
where
    T: TxRules,
    Q: Queryable,
{
    pub action: T,
    pub query_type: Q,
}

/// A trait that contains information about a link. It could be distance, load, etc.
pub trait LinkFeatures: Copy + Clone + Debug + Default {}

/// A struct that represents a link between two nodes defined by the features F.
#[derive(Debug, Copy, Clone, Default, TypedBuilder)]
pub struct GLink<F>
where
    F: LinkFeatures,
{
    pub target: NodeId,
    pub properties: F,
}

impl<F> GLink<F>
where
    F: LinkFeatures,
{
    pub fn new(target: NodeId) -> Self {
        Self {
            target,
            properties: F::default(),
        }
    }
}

/// A trait that represents a radio that can be used to transfer data. It performs the actual
/// data transfer and can be used to measure the radio usage.
pub trait Channel<M, N, T>
where
    M: Metadata,
    N: NodeState,
    T: TxRules,
{
    fn reset(&mut self);
    fn do_transfers(&mut self, payloads: Vec<GPayload<M, N>>) -> Vec<GPayload<M, N>>;
    fn apply_tx_rules(
        &mut self,
        tx_rules: &T,
        payloads: Vec<GPayload<M, N>>,
    ) -> Vec<GPayload<M, N>>;
}

/// A trait to represent a type that holds statistics of the radio usage for incoming data.
pub trait IncomingStats<M>: Clone + Copy + Debug
where
    M: Metadata,
{
    fn add_attempted(&mut self, metadata: &M);
    fn add_feasible(&mut self, metadata: &M);
}

/// A trait to represent a type that holds statistics of the radio usage for outgoing data.
pub trait OutgoingStats<M>: Clone + Copy + Debug
where
    M: Metadata,
{
    fn update(&mut self, metadata: &M);
}
