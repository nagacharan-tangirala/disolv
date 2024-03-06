use typed_builder::TypedBuilder;

/// A trait to represent a type that can be used to query content from other devices.
pub trait Queryable: Copy + Clone + PartialEq + Eq + Send + Sync {}

/// A trait to represent a type that can be used to represent the individual content of
/// a payload. Extend this to a custom type (e.g. struct) that you want to use as a collection
/// of data that is being transferred by a device.
pub trait DataUnit: Clone + Send + Sync {}

/// A trait that represents the agent state. Extend this to a custom type (e.g. struct)
/// that you want to use to represent agent's state.
pub trait AgentState: Copy + Clone + Send + Sync {}

/// A trait that represents the metadata of a payload. Extend this to a custom type (e.g. struct)
/// that contains the metadata such as the size, count, etc. of a payload. The struct extending
/// this trait must contain information that is useful to evaluate if the transmission is feasible.
/// It should contain information about the queryable content of the payload.
pub trait Metadata: Clone + Send + Sync {}

/// A generic struct that represents a payload of a device. A message exchange between two devices
/// can be represented by a payload. Gathered content can be used to represent the aggregated
/// content from the downstream devices that require forwarding. A payload is a combination of
/// the agent state, metadata, and gathered states.
#[derive(Clone, Debug, Default, TypedBuilder)]
pub struct GPayload<A, M>
where
    A: AgentState,
    M: Metadata,
{
    pub agent_state: A,
    pub metadata: M,
    pub gathered_states: Option<Vec<A>>,
}

/// A trait to indicate a type that can be used to represent the transfer status of a payload.
pub trait PayloadStatus: Clone + Send + Sync {}

/// A trait to indicate a type that can be used to convey the payload transfer status back
/// to the device that sent the payload.
///
/// Metadata can contain transfer metrics such as the status, latency, etc.
pub trait TxReport: Clone + Send + Sync {}

/// A trait to indicate a type that can be used to represent the content of a response. The content
/// can contain queries that can be read by other devices.
pub trait Reply: Clone + Send + Sync {}

/// A generic struct that represents a response from a device. A response can be used to relay
/// queries and payload transfer metrics to other devices after they send a payload.
///
/// Queries can be optionally included in the response to control the content that is
/// being transferred.
#[derive(Clone, Debug, Default, TypedBuilder)]
pub struct GResponse<R, T>
where
    R: Reply,
    T: TxReport,
{
    pub reply: Option<R>,
    pub downstream: Option<Vec<R>>,
    pub tx_report: T,
}
