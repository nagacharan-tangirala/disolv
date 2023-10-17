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
pub struct PayloadData<D, I, Q>
where
    D: DataCreek<Q>,
    I: Identifier,
    Q: Queryable,
{
    pub data_pile: D,
    pub from_node: I,
    _phantom: std::marker::PhantomData<fn() -> Q>,
}

impl<I, D, Q> PayloadData<D, I, Q>
where
    D: DataCreek<Q>,
    I: Identifier,
    Q: Queryable,
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
pub struct Payload<D, I, P, Q>
where
    D: DataCreek<Q>,
    I: Identifier,
    P: PayloadStats<D, Q>,
    Q: Queryable,
{
    pub gathered_data: Option<Vec<PayloadData<D, I, Q>>>,
    pub data_pile: PayloadData<D, I, Q>,
    pub payload_stats: P,
    pub _phantom: std::marker::PhantomData<fn() -> (D, I, Q)>,
}

impl<D, I, P, Q> Payload<D, I, P, Q>
where
    D: DataCreek<Q>,
    I: Identifier,
    P: PayloadStats<D, Q>,
    Q: Queryable,
{
    pub fn new(data_pile: PayloadData<D, I, Q>, payload_stats: P) -> Self {
        Self {
            gathered_data: None,
            data_pile,
            payload_stats,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn add_gathered_data(&mut self, gathered_data: Option<Vec<PayloadData<D, I, Q>>>) {
        self.gathered_data = gathered_data;
    }
}
