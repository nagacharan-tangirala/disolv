use crate::bucket::{Bucket, TimeS};
use crate::engine::Engine;
use crate::entity::{Entity, Identifier, Kind};
use krabmaga::engine::agent::Agent;
use krabmaga::engine::state::State;
use std::fmt;
use std::fmt::Display;
use std::hash::{Hash, Hasher};

#[derive(Clone, Default)]
pub struct Node<I, N, K, B, S>
where
    I: Identifier,
    N: Entity<B, S>,
    K: Kind,
    B: Bucket<S>,
    S: TimeS,
{
    pub(crate) node_id: I,
    pub(crate) node: N,
    pub(crate) kind: K,
    _marker: std::marker::PhantomData<fn() -> (B, S)>,
}

impl<I, N, K, B, S> Agent for Node<I, N, K, B, S>
where
    I: Identifier,
    N: Entity<B, S>,
    K: Kind + Display,
    B: Bucket<S>,
    S: TimeS,
{
    fn step(&mut self, state: &mut dyn State) {
        let engine: &mut Engine<K, B, S> = state
            .as_any_mut()
            .downcast_mut::<Engine<K, B, S>>()
            .unwrap();
        let bucket: &mut B = match engine.bucket_of(self.kind) {
            Some(bucket) => bucket,
            None => panic!("Could not find bucket for type {}", self.kind),
        };
        self.node.step(bucket);
    }

    fn after_step(&mut self, state: &mut dyn State) {
        let engine = state
            .as_any_mut()
            .downcast_mut::<Engine<K, B, S>>()
            .unwrap();
        let bucket = match engine.bucket_of(self.kind) {
            Some(bucket) => bucket,
            None => panic!("Could not find bucket for type {}", self.kind),
        };
        self.node.after_step(bucket);
    }

    fn is_stopped(&self, _state: &mut dyn State) -> bool {
        self.node.is_stopped()
    }
}

impl<I, N, K, B, S> Node<I, N, K, B, S>
where
    I: Identifier,
    N: Entity<B, S>,
    K: Kind,
    B: Bucket<S>,
    S: TimeS,
{
    pub fn new(node_id: I, node: N, kind: K) -> Self {
        Self {
            node_id,
            node,
            kind,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<I, N, K, B, S> Hash for Node<I, N, K, B, S>
where
    I: Identifier,
    N: Entity<B, S>,
    K: Kind,
    B: Bucket<S>,
    S: TimeS,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.node_id.hash(state);
    }
}

impl<I, N, K, B, S> Display for Node<I, N, K, B, S>
where
    I: Identifier + Display,
    N: Entity<B, S>,
    K: Kind,
    B: Bucket<S>,
    S: TimeS,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.node_id)
    }
}

impl<I, N, K, B, S> Eq for Node<I, N, K, B, S>
where
    I: Identifier,
    N: Entity<B, S>,
    K: Kind,
    B: Bucket<S>,
    S: TimeS,
{
}

impl<I, N, K, B, S> PartialEq for Node<I, N, K, B, S>
where
    I: Identifier,
    N: Entity<B, S>,
    K: Kind,
    B: Bucket<S>,
    S: TimeS,
{
    fn eq(&self, other: &Node<I, N, K, B, S>) -> bool {
        self.node_id == other.node_id
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::bucket::tests::{MyBucket, Ts};
    use crate::entity::tests::{make_device, DeviceType, Nid, TDevice};
    use crate::node::Node;

    pub(crate) type MyNode = Node<Nid, TDevice, DeviceType, MyBucket, Ts>;

    pub(crate) fn as_node(device: TDevice) -> MyNode {
        Node::new(device.id, device, device.device_type)
    }

    #[test]
    fn test_make_nodes() {
        let device_a = make_device(Nid::from(1), DeviceType::TypeA, 1);
        let node_a = as_node(device_a);
        assert_eq!(node_a.node_id, Nid::from(1));
        assert_eq!(node_a.kind, DeviceType::TypeA);
    }
}
