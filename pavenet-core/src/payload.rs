use crate::response::Queryable;
use pavenet_engine::entity::Identifier;

pub trait DataCreek<Q>: Copy + Clone + Send + Sync
where
    Q: Queryable,
{
}

pub trait PayloadStats<D, Q>: Clone + Send + Sync
where
    D: DataCreek<Q>,
    Q: Queryable,
{
}

#[derive(Clone, Default)]
pub struct PayloadData<I, D, Q>
where
    I: Identifier,
    Q: Queryable,
    D: DataCreek<Q>,
{
    pub data_pile: D, // consolidate after gathering
    pub from_node: I,
    _phantom: std::marker::PhantomData<fn() -> Q>,
}

impl<I, D, Q> PayloadData<I, D, Q>
where
    I: Identifier,
    Q: Queryable,
    D: DataCreek<Q>,
{
    pub fn new(data_pile: D, from_node: I) -> Self {
        Self {
            data_pile,
            from_node,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[derive(Clone, Default)]
pub struct Payload<D, P, I, Q>
where
    D: DataCreek<Q>,
    P: PayloadStats<D, Q>,
    I: Identifier,
    Q: Queryable,
{
    pub gathered_data: Option<Vec<PayloadData<I, D, Q>>>,
    pub data_pile: PayloadData<I, D, Q>,
    pub payload_stats: P,
    pub _phantom: std::marker::PhantomData<fn() -> (D, I, Q)>,
}
