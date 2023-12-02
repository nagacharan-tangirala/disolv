use crate::message::{GPayload, Metadata, NodeState};
use crate::node::NodeId;
use std::fmt::Debug;
use typed_builder::TypedBuilder;

/// Use this trait to mark a type as a rule. This trait is used to enforce rules
/// on the payload transmission.
pub trait Action: Default + Copy + Clone + Send + Sync {}

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
pub trait Channel<M, N>
where
    M: Metadata,
    N: NodeState,
{
    fn reset(&mut self);
    fn do_transfers(&mut self, payloads: Vec<GPayload<M, N>>) -> Vec<GPayload<M, N>>;
    fn update_actions(
        &mut self,
        node_state: &N,
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
