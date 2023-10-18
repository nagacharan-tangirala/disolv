use pavenet_engine::bucket::{Bucket, TimeStamp};
use pavenet_engine::entity::Identifier;

pub trait Queryable: Copy + Clone + Send + Sync {}

pub trait RequestCreek<Q>: Clone + Send + Sync {}

#[derive(Clone, Default)]
pub struct RequestData<I, Q, R>
where
    I: Identifier,
    Q: Queryable,
    R: RequestCreek<Q>,
{
    pub request: R,
    pub request_to: I,
    _phantom: std::marker::PhantomData<fn() -> Q>,
}

impl<I, Q, R> RequestData<I, Q, R>
where
    I: Identifier,
    Q: Queryable,
    R: RequestCreek<Q>,
{
    pub fn new(request: R, request_to: I) -> Self {
        Self {
            request,
            request_to,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub trait TransferStats: Clone + Send + Sync {}

#[derive(Clone, Default)]
pub struct Response<I, Q, R, T>
where
    I: Identifier,
    Q: Queryable,
    R: RequestCreek<Q>,
    T: TransferStats,
{
    pub relayed_requests: Option<Vec<RequestData<I, Q, R>>>,
    pub transfer_stats: T,
    _phantom: std::marker::PhantomData<fn() -> (I, Q)>,
}

impl<I, Q, R, T> Response<I, Q, R, T>
where
    I: Identifier,
    Q: Queryable,
    R: RequestCreek<Q>,
    T: TransferStats,
{
    pub fn new(transfer_stats: T) -> Self {
        Self {
            transfer_stats,
            relayed_requests: None,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub trait Downloader<B, I, Q, R, T, TS>
where
    B: Bucket<T>,
    I: Identifier,
    Q: Queryable,
    R: RequestCreek<Q>,
    T: TimeStamp,
    TS: TransferStats,
{
    fn fetch_feedback(&mut self, bucket: &mut B) -> Option<Response<I, Q, R, TS>>;
    fn build_responses(&mut self, bucket: &mut B) -> Option<Vec<Response<I, Q, R, TS>>>;
    fn relay_responses(&mut self, responses: Option<Vec<Response<I, Q, R, TS>>>, bucket: B);
}
