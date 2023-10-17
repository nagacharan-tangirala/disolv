use pavenet_engine::entity::{Identifier, Kind};

pub trait LinkInfo<I> {}

pub struct NodeLink<I, L>
where
    I: Identifier,
    L: LinkInfo<I>,
{
    pub target: Vec<I>,
    pub link_info: L,
}

impl<I, L> NodeLink<I, L>
where
    I: Identifier,
    L: LinkInfo<I>,
{
    pub fn new(target: Vec<I>, link_info: L) -> Self {
        Self { target, link_info }
    }
}

trait NodeFinder<I, K, L>
where
    I: Identifier,
    K: Kind,
    L: LinkInfo<I>,
{
    fn nodes_of_kind(&self, kind: K) -> Option<&Vec<NodeLink<I, L>>>;
}
