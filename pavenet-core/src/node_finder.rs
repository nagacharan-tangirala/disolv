use pavenet_engine::entity::{Identifier, Kind};

trait NodeFinder<K, I>
where
    K: Kind,
    I: Identifier,
{
    fn nodes_of_kind(&self, kind: K) -> Option<&Vec<I>>;
}
