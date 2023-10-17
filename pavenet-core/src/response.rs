use pavenet_engine::entity::Identifier;

pub trait Queryable: Copy + Clone + Send + Sync {}

pub trait RequestCreek<Q>: Clone + Send + Sync {}

#[derive(Clone, Default)]
pub struct FeedbackData<Q, I, R>
where
    Q: Queryable,
    R: RequestCreek<Q>,
    I: Identifier,
{
    pub feedback: R,
    pub feedback_to: I,
    _phantom: std::marker::PhantomData<fn() -> Q>,
}

pub trait TransferStats: Clone + Send + Sync {}

#[derive(Clone, Default)]
pub struct Response<Q, I, R, T>
where
    Q: Queryable,
    R: RequestCreek<Q>,
    I: Identifier,
    T: TransferStats,
{
    pub feedbacks: Option<Vec<FeedbackData<Q, I, R>>>,
    pub transfer_stats: T,
    _phantom: std::marker::PhantomData<fn() -> (Q, I)>,
}
