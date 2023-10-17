use crate::payload::{DataCreek, Payload, PayloadStats};
use crate::response::Queryable;
use pavenet_engine::bucket::{Bucket, TimeStamp};
use pavenet_engine::entity::Identifier;

pub trait DataMaker<D, I, P, Q>
where
    D: DataCreek<Q>,
    I: Identifier,
    P: PayloadStats<D, Q>,
    Q: Queryable,
{
    fn make_payload(&mut self) -> Payload<D, I, P, Q>;
    fn update_payload(
        &mut self,
        given: &mut Payload<D, I, P, Q>,
        incoming: Option<Vec<Payload<D, I, P, Q>>>,
    );
}

pub trait Uploader<B, D, I, P, Q, T>
where
    B: Bucket<T>,
    D: DataCreek<Q>,
    I: Identifier,
    P: PayloadStats<D, Q>,
    Q: Queryable,
    T: TimeStamp,
{
    fn gather(&mut self, bucket: &mut B) -> Option<Vec<Payload<D, I, P, Q>>>;
    fn transmit(&mut self, payload: Payload<D, I, P, Q>, bucket: &mut B);
}
