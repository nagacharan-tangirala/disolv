use crate::response::{Queryable, RequestCreek, RequestData, Response, TransferStats};
use pavenet_engine::bucket::{Bucket, TimeStamp};
use pavenet_engine::entity::Identifier;

pub trait ResponseMaker<B, I, Q, R, T, TS>
where
    B: Bucket<T>,
    I: Identifier,
    Q: Queryable,
    R: RequestCreek<Q>,
    T: TimeStamp,
    TS: TransferStats,
{
    fn read_response(&mut self, response: Response<I, Q, R, TS>);
    fn build_responses(&mut self, bucket: &mut B) -> Option<Vec<Response<I, Q, R, TS>>>;
}

pub trait Downloader<Q, T, TS, I, R, B>
where
    R: RequestCreek<Q>,
    I: Identifier,
    T: TimeStamp,
    TS: TransferStats,
    B: Bucket<T>,
    Q: Queryable,
{
    fn fetch_feedback(&mut self, bucket: &mut B) -> Option<Response<I, Q, R, TS>>;
    fn send_response(&mut self, responses: Option<Vec<Response<I, Q, R, TS>>>, bucket: B);
}
