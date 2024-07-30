use std::fmt::Display;
use std::hash::Hash;
use std::marker::PhantomData;

use typed_builder::TypedBuilder;

use crate::agent::AgentProperties;
use crate::metrics::Bytes;
use crate::radio::{Action, ActionType};

/// A trait to represent a type that can be used to query content from other devices.
pub trait QueryType: Copy + Clone + Default + PartialEq + Eq + Send + Sync {}

/// A trait to represent a type of data that each query contains.
pub trait ContentType:
    Display + Copy + Clone + Default + Hash + PartialEq + Eq + Send + Sync
{
}

/// A trait to represent a type that can be used to represent the individual content of
/// a payload. Extend this to a custom type (e.g. struct) that you want to use as a collection
/// of data that is being transferred by a device.
pub trait DataUnit<C>: Clone + Send + Sync
where
    C: ContentType,
{
    fn action(&self) -> &Action;
    fn content_type(&self) -> &C;
    fn size(&self) -> Bytes;
    fn update_action(&mut self, new_action: &Action);
}

/// A trait that represents the metadata of a payload. Extend this to a custom type (e.g. struct)
/// that contains the metadata such as the size, count, etc. of a payload. The struct extending
/// this trait must contain information that is useful to evaluate if the transmission is feasible.
/// It should contain information about the queryable content of the payload.
pub trait Metadata: Clone + Send + Sync {
    fn size(&self) -> Bytes;
    fn count(&self) -> u32;
    fn set_size(&mut self, bytes: Bytes);
    fn set_count(&mut self, count: u32);
}

/// A generic struct that represents a payload of a device. A message exchange between two devices
/// can be represented by a payload. Gathered content can be used to represent the aggregated
/// content from the downstream devices that require forwarding. A payload is a combination of
/// the agent state, metadata, and gathered states.
#[derive(Clone, Debug, Default, TypedBuilder)]
pub struct Payload<C, D, M, P, Q>
where
    C: ContentType,
    D: DataUnit<C>,
    M: Metadata,
    P: AgentProperties,
    Q: QueryType,
{
    pub query_type: Q,
    pub agent_state: P,
    pub metadata: M,
    pub data_units: Vec<D>,
    pub gathered_states: Option<Vec<P>>,
    #[builder(default)]
    _phantom_data: PhantomData<fn() -> C>,
}

impl<C, D, M, P, Q> Payload<C, D, M, P, Q>
where
    C: ContentType,
    D: DataUnit<C>,
    M: Metadata,
    P: AgentProperties,
    Q: QueryType,
{
    pub fn consume(&mut self) {
        // Get the current size and count
        let mut total_size = self.metadata.size();
        let mut total_count = self.metadata.count();

        // Remove stats of the data units marked as consume
        self.data_units.iter_mut().for_each(|unit| {
            if unit.action().action_type == ActionType::Consume {
                total_size -= unit.size();
                total_count -= 1;
            }
        });

        // Update the metadata with the new size and count.
        self.metadata.set_size(total_size);
        self.metadata.set_count(total_count);

        // Remove the data units marked as consume
        self.data_units
            .retain(|unit| unit.action().action_type != ActionType::Consume);
    }
}

/// A trait to indicate a type that can be used to represent the transfer status of a payload.
pub trait PayloadStatus: Clone + Send + Sync {}

/// A trait to indicate a type that can be used to convey the payload transfer status back
/// to the device that sent the payload.
///
/// Metadata can contain transfer metrics such as the status, latency, etc.
pub trait TxReport: Clone + Send + Sync {}
