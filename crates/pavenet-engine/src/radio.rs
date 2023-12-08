use crate::entity::Class;
use crate::message::{DataUnit, GPayload, Metadata, NodeState, RxReport};
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
pub trait RxChannel<M, N>
where
    M: Metadata,
    N: NodeState,
{
    type R: RxReport;

    fn reset_rx(&mut self);
    fn complete_transfers(
        &mut self,
        payloads: Vec<GPayload<M, N>>,
    ) -> (Vec<GPayload<M, N>>, Vec<Self::R>);
    fn perform_actions(
        &mut self,
        node_state: &N,
        payloads: Vec<GPayload<M, N>>,
    ) -> Vec<GPayload<M, N>>;
}

/// A trait that represents a channel that can be used to transmit data. It prepares the data
/// for transmission and applies the actions to the data blobs before transmission.
pub trait TxChannel<M, N>
where
    M: Metadata,
    N: NodeState,
{
    type C: Class;
    type D: DataUnit;

    fn reset(&mut self);
    fn prepare_blobs_to_fwd(
        &mut self,
        target_class: &N,
        to_forward: &Vec<GPayload<M, N>>,
    ) -> Vec<Self::D>;
    fn prepare_transfer(
        &mut self,
        target_class: &Self::C,
        payload: GPayload<M, N>,
    ) -> GPayload<M, N>;
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
