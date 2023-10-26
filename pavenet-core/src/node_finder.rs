use pavenet_engine::entity::{Identifier, Kind};

pub trait LinkFeatures {}

trait NodeFinder<I, K, L>
where
    I: Identifier,
    K: Kind,
    L: LinkFeatures,
{
    fn link_for(&self, kind: K) -> Option<&Vec<LinkOptions<I, L>>>;
}
