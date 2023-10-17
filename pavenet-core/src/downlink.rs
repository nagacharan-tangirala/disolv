use crate::response::{FeedbackData, Queryable, RequestCreek, Response, TransferStats};
use pavenet_engine::bucket::{Bucket, TimeStamp};
use pavenet_engine::entity::Identifier;

pub trait Receiver<B, R, I, T, Q>
where
    B: Bucket<T>,
    R: RequestCreek<Q>,
    I: Identifier,
    Q: Queryable,
    T: TimeStamp,
{
    fn fetch_feedbacks(&mut self, bucket: B) -> Option<Vec<FeedbackData<Q, I, R>>>;
}

pub trait Responder<Q, T, TS, I, R, B>
where
    R: RequestCreek<Q>,
    I: Identifier,
    T: TimeStamp,
    TS: TransferStats,
    B: Bucket<T>,
    Q: Queryable,
{
    fn response(&mut self) -> Response<Q, I, R, TS>;
    fn send_response(&mut self, bucket: B);
}
