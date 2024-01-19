use crate::bucket::Bucket;
use crate::entity::Class;
use crate::message::{
    DataUnit, GPayload, GResponse, Metadata, NodeState, Queryable, Reply, TxReport,
};
use crate::node::NodeId;
use std::fmt::Debug;
use typed_builder::TypedBuilder;

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

/// Use this trait to mark a type as an action. This can be used to define custom actions
/// that can be performed on a payload in the form of enum to indicate actions such as
/// consume, forward, etc.
pub trait Actionable: Default + Copy + Clone + Send + Sync {}

/// A trait that contains information that can assist in performing an action on a payload.
/// Use this on a struct that contains information about the action to be performed.
/// For example, the action can be to forward the payload to a specific node or class.
pub trait ActionInfo: Copy + Clone + Send + Sync {}

/// A struct that represents an action that can be performed on a payload.
/// Action can be different for different data types.
#[derive(Debug, Clone, Default)]
pub struct Actions<I, Q>
where
    I: ActionInfo,
    Q: Queryable,
{
    pub data_type: Vec<Q>,
    pub action_info: Vec<I>,
}

impl<I, Q> Actions<I, Q>
where
    I: ActionInfo,
    Q: Queryable,
{
    pub fn add_action(&mut self, data_type: Q, action_info: I) {
        self.data_type.push(data_type);
        self.action_info.push(action_info);
    }

    pub fn action_for(&self, data_type: &Q) -> Option<&I> {
        self.data_type
            .iter()
            .zip(self.action_info.iter())
            .find(|(dt, _)| *dt == data_type)
            .map(|(_, ai)| ai)
    }
}

/// A trait that an entity must implement to transmit payloads. Transmission of payloads
/// can be flexibly handled by the entity and can transfer payloads to devices of any tier.
/// This should be called in the <code>uplink_stage</code> method of the entity.
pub trait Transmitter<B, F, M, N>
where
    B: Bucket,
    F: LinkFeatures,
    M: Metadata,
    N: NodeState,
{
    type NodeClass: Class;

    fn transmit(&mut self, payload: GPayload<M, N>, target: GLink<F>, bucket: &mut B);
    fn transmit_sl(&mut self, payload: GPayload<M, N>, target: GLink<F>, bucket: &mut B);
}

/// A trait that an entity must implement to receive messages from other entities in the
/// simulation. The messages can be from the same class or from up/downstream.
pub trait Receiver<B, M, N>
where
    B: Bucket,
    M: Metadata,
    N: NodeState,
{
    type C: Class;

    fn receive(&mut self, bucket: &mut B) -> Option<Vec<GPayload<M, N>>>;
    fn receive_sl(&mut self, bucket: &mut B) -> Option<Vec<GPayload<M, N>>>;
}

/// A trait that an entity must implement to respond to payloads. Transmission of payloads
/// can be flexibly handled by the entity transfer payloads to devices of any tier.
/// This should be called in the <code>downlink_stage</code> method of the entity.
pub trait Responder<B, R, T>
where
    B: Bucket,
    R: Reply,
    T: TxReport,
{
    fn respond(&mut self, response: Option<GResponse<R, T>>, bucket: &mut B);
    fn respond_sl(&mut self, response: Option<GResponse<R, T>>, bucket: &mut B);
}
