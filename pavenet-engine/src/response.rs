use crate::bucket::Bucket;
use crate::entity::Tier;

/// A trait to represent a type that can be used to query content from other devices.
pub trait Queryable: Copy + Clone + PartialEq + Eq + Send + Sync {}

/// A trait to indicate a type that can be used represent the content of a response. The content
/// can contain queries that can be read by other devices.
pub trait ResponseContent<Q>: Clone + Send + Sync {}

/// A trait to indicate a type that can be used to convey the payload transfer status back
/// to the device that sent the payload.
///
/// Metadata can contain transfer metrics such as the status, latency, etc.
pub trait ResponseMetadata: Clone + Send + Sync {}

/// A generic struct that represents a response from a device. A response can be used to relay
/// queries and payload transfer metrics to other devices after they send a payload.
///
/// Queries can be optionally included in the response to control the content that is
/// being transferred.
#[derive(Clone, Debug, Default)]
pub struct GResponse<C, M, Q>
where
    C: ResponseContent<Q>,
    M: ResponseMetadata,
    Q: Queryable,
{
    pub content: Option<Vec<C>>,
    pub transfer_stats: M,
    _phantom: std::marker::PhantomData<fn() -> Q>,
}

impl<C, M, Q> GResponse<C, M, Q>
where
    C: ResponseContent<Q>,
    M: ResponseMetadata,
    Q: Queryable,
{
    pub fn new(transfer_stats: M, content: Option<Vec<C>>) -> Self {
        Self {
            transfer_stats,
            content,
            _phantom: std::marker::PhantomData,
        }
    }
}

/// A trait that an entity must implement to respond to payloads. Transmission of payloads
/// can be flexibly handled by the entity transfer payloads to devices of any tier.
/// This should be called in the <code>downlink_stage</code> method of the entity.
pub trait Responder<B, C, M, Q, T, Ts>
where
    B: Bucket,
    C: ResponseContent<Q>,
    M: ResponseMetadata,
    Q: Queryable,
    T: Tier,
{
    fn receive(&mut self, bucket: &mut B) -> Option<GResponse<C, M, Q>>;
    fn respond(&mut self, response: Option<GResponse<C, M, Q>>, bucket: &mut B);
}
