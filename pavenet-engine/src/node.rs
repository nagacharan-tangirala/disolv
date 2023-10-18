use crate::bucket::{Bucket, TimeStamp};
use crate::engine::Engine;
use crate::entity::{Entity, Identifier, Kind};
use krabmaga::engine::agent::Agent;
use krabmaga::engine::state::State;
use std::fmt;
use std::fmt::Display;
use std::hash::{Hash, Hasher};

#[derive(Clone, Default)]
pub struct Node<B, I, K, N, S>
where
    B: Bucket<S>,
    I: Identifier,
    K: Kind,
    N: Entity<B, S>,
    S: TimeStamp,
{
    pub node_id: I,
    pub node: N,
    pub kind: K,
    _marker: std::marker::PhantomData<fn() -> (B, S)>,
}

impl<B, I, K, N, S> Agent for Node<B, I, K, N, S>
where
    B: Bucket<S>,
    I: Identifier,
    K: Kind,
    N: Entity<B, S>,
    S: TimeStamp,
{
    fn step(&mut self, state: &mut dyn State) {
        let engine: &mut Engine<B, S> = state.as_any_mut().downcast_mut::<Engine<B, S>>().unwrap();
        self.node.step(&mut engine.bucket);
    }

    fn after_step(&mut self, state: &mut dyn State) {
        let engine = state.as_any_mut().downcast_mut::<Engine<B, S>>().unwrap();
        self.node.after_step(&mut engine.bucket);
    }

    fn is_stopped(&self, _state: &mut dyn State) -> bool {
        self.node.is_stopped()
    }
}

impl<B, I, K, N, S> Node<B, I, K, N, S>
where
    B: Bucket<S>,
    I: Identifier,
    K: Kind,
    N: Entity<B, S>,
    S: TimeStamp,
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

impl<B, I, K, N, S> Hash for Node<B, I, K, N, S>
where
    B: Bucket<S>,
    I: Identifier,
    K: Kind,
    N: Entity<B, S>,
    S: TimeStamp,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.node_id.hash(state);
    }
}

impl<B, I, K, N, S> Display for Node<B, I, K, N, S>
where
    B: Bucket<S>,
    I: Identifier,
    K: Kind,
    N: Entity<B, S>,
    S: TimeStamp,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.node_id)
    }
}

impl<B, I, K, N, S> Eq for Node<B, I, K, N, S>
where
    B: Bucket<S>,
    I: Identifier,
    K: Kind,
    N: Entity<B, S>,
    S: TimeStamp,
{
}

impl<B, I, K, N, S> PartialEq for Node<B, I, K, N, S>
where
    B: Bucket<S>,
    I: Identifier,
    K: Kind,
    N: Entity<B, S>,
    S: TimeStamp,
{
    fn eq(&self, other: &Node<B, I, K, N, S>) -> bool {
        self.node_id == other.node_id
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::bucket::tests::{MyBucket, Ts};
    use crate::entity::tests::{make_device, DeviceType, Nid, TDevice};
    use crate::node::Node;

    pub(crate) type MyNode = Node<MyBucket, Nid, DeviceType, TDevice, Ts>;

    pub(crate) fn as_node(device: TDevice) -> MyNode {
        let device_type = device.device_type.clone();
        Node::new(device.id, device, device_type)
    }

    #[test]
    fn test_make_nodes() {
        let device_a = make_device(Nid::from(1), DeviceType::TypeA, 1);
        let node_a = as_node(device_a);
        assert_eq!(node_a.node_id, Nid::from(1));
        assert_eq!(node_a.kind, DeviceType::TypeA);
    }
}
