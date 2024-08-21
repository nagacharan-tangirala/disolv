use std::fmt::Debug;

use serde::Deserialize;
use typed_builder::TypedBuilder;

use crate::agent::{AgentClass, AgentId, AgentKind};
use crate::agent::AgentProperties;
use crate::bucket::Bucket;
use crate::message::{ContentType, DataUnit, Metadata, Payload, QueryType};

/// A trait that contains information about a link. It could be distance, load, etc.
pub trait LinkFeatures: Copy + Clone + Debug + Default {}

/// A struct that represents a link between two agents defined by the features F.
#[derive(Debug, Copy, Clone, Default, TypedBuilder)]
pub struct Link<F>
where
    F: LinkFeatures,
{
    pub target: AgentId,
    pub properties: F,
}

impl<F> Link<F>
where
    F: LinkFeatures,
{
    pub fn new(target: AgentId) -> Self {
        Self {
            target,
            properties: F::default(),
        }
    }
}

/// The type of actions each data unit must be set to. This guides the devices in assisting
/// what to do with each piece of information received from the neighbours.
#[derive(Deserialize, Clone, Debug, Copy, Eq, PartialEq, Default)]
pub enum ActionType {
    #[default]
    Consume,
    Forward,
    Fl,
}

/// A generic action struct that can be used to contain the information about action to perform
/// on a given message.
#[derive(Clone, Default, Debug, TypedBuilder)]
pub struct Action {
    pub action_type: ActionType,
    pub to_class: Option<AgentClass>,
    pub to_agent: Option<AgentId>,
    pub to_kind: Option<AgentKind>,
    pub to_broadcast: Option<Vec<AgentId>>,
}

impl Action {
    pub fn with_action_type(action_type: ActionType) -> Self {
        Self {
            action_type,
            to_class: None,
            to_agent: None,
            to_kind: None,
            to_broadcast: None,
        }
    }
}

/// A trait that an entity must implement to transmit payloads. Transmission of payloads
/// can be flexibly handled by the entity and can transfer payloads to devices of any tier.
/// This should be called in the <code>uplink_stage</code> method of the entity.
pub trait Transmitter<B, C, D, F, M, P, Q>
where
    B: Bucket,
    C: ContentType,
    D: DataUnit<C>,
    F: LinkFeatures,
    M: Metadata,
    P: AgentProperties,
    Q: QueryType,
{
    fn transmit(&mut self, payload: Payload<C, D, M, P, Q>, target: Link<F>, bucket: &mut B);
    fn transmit_sl(&mut self, payload: Payload<C, D, M, P, Q>, target: Link<F>, bucket: &mut B);
}

/// A trait that an entity must implement to receive messages from other entities in the
/// simulation. The messages can be from the same class or from up/downstream.
pub trait Receiver<B, C, D, M, P, Q>
where
    B: Bucket,
    C: ContentType,
    D: DataUnit<C>,
    M: Metadata,
    P: AgentProperties,
    Q: QueryType,
{
    fn receive(&mut self, bucket: &mut B) -> Option<Vec<Payload<C, D, M, P, Q>>>;
    fn receive_sl(&mut self, bucket: &mut B) -> Option<Vec<Payload<C, D, M, P, Q>>>;
}
