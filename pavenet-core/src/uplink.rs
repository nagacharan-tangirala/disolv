use crate::payload::{DataCreek, Payload, PayloadData, PayloadStats};
use crate::response::Queryable;
use pavenet_engine::bucket::{Bucket, TimeStamp};
use pavenet_engine::entity::Identifier;

pub trait Gatherer<T, I, D, B, Q>
where
    I: Identifier,
    D: DataCreek<Q>,
    T: TimeStamp,
    B: Bucket<T>,
    Q: Queryable,
{
    fn gather(&mut self, bucket: &mut B) -> Option<Vec<PayloadData<I, D, Q>>>;
}

pub trait DataMaker<I, D, Q, P>
where
    D: DataCreek<Q>,
    Q: Queryable,
    I: Identifier,
    P: PayloadStats<D, Q>,
{
    fn make_data(&mut self) -> PayloadData<I, D, Q>;
    fn payload_stats(&mut self) -> P;
}

pub trait Transmitter<B, D, I, P, T, Q>
where
    D: DataCreek<Q>,
    Q: Queryable,
    P: PayloadStats<D, Q>,
    I: Identifier,
    T: TimeStamp,
    B: Bucket<T>,
{
    fn payload(&mut self) -> Payload<D, P, I, Q>;
    fn transmit(&mut self, bucket: &mut B);
}
