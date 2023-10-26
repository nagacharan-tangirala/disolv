use crate::bucket::{Bucket, TimeStamp};
use crate::engine::Engine;
use crate::entity::{Entity, Identifier, Kind, Tier};
use crate::scheduler::NodeScheduler;
use krabmaga::engine::agent::Agent;
use krabmaga::engine::state::State;
use std::fmt;
use std::fmt::Display;
use std::hash::{Hash, Hasher};

#[derive(Clone, Default)]
pub struct Node<B, E, I, K, T, Ts>
where
    B: Bucket<Ts>,
    E: Entity<B, T, Ts>,
    I: Identifier,
    K: Kind,
    T: Tier,
    Ts: TimeStamp,
{
    pub node_id: I,
    pub entity: E,
    pub kind: K,
    _marker: std::marker::PhantomData<fn() -> (B, T, Ts)>,
}

impl<B, E, I, K, T, Ts> Agent for Node<B, E, I, K, T, Ts>
where
    B: Bucket<Ts>,
    E: Entity<B, T, Ts>,
    I: Identifier,
    K: Kind,
    T: Tier,
    Ts: TimeStamp,
{
    fn step(&mut self, state: &mut dyn State) {
        let engine: &mut Engine<B, NodeScheduler<B, E, I, K, T, Ts>, Ts> = state
            .as_any_mut()
            .downcast_mut::<Engine<B, NodeScheduler<B, E, I, K, T, Ts>, Ts>>()
            .unwrap();
        self.entity.uplink_stage(&mut engine.bucket);
    }

    fn after_step(&mut self, state: &mut dyn State) {
        let engine = state
            .as_any_mut()
            .downcast_mut::<Engine<B, NodeScheduler<B, E, I, K, T, Ts>, Ts>>()
            .unwrap();
        self.entity.downlink_stage(&mut engine.bucket);
    }

    fn is_stopped(&self, _state: &mut dyn State) -> bool {
        self.entity.is_stopped()
    }
}

impl<B, E, I, K, T, Ts> Node<B, E, I, K, T, Ts>
where
    B: Bucket<Ts>,
    E: Entity<B, T, Ts>,
    I: Identifier,
    K: Kind,
    T: Tier,
    Ts: TimeStamp,
{
    pub fn new(node_id: I, node: E, kind: K) -> Self {
        Self {
            node_id,
            entity: node,
            kind,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<B, E, I, K, T, Ts> Hash for Node<B, E, I, K, T, Ts>
where
    B: Bucket<Ts>,
    E: Entity<B, T, Ts>,
    I: Identifier,
    K: Kind,
    T: Tier,
    Ts: TimeStamp,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.node_id.hash(state);
    }
}

impl<B, E, I, K, T, Ts> Display for Node<B, E, I, K, T, Ts>
where
    B: Bucket<Ts>,
    E: Entity<B, T, Ts>,
    I: Identifier,
    K: Kind,
    T: Tier,
    Ts: TimeStamp,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.node_id)
    }
}

impl<B, E, I, K, T, Ts> PartialEq for Node<B, E, I, K, T, Ts>
where
    B: Bucket<Ts>,
    E: Entity<B, T, Ts>,
    I: Identifier,
    K: Kind,
    T: Tier,
    Ts: TimeStamp,
{
    fn eq(&self, other: &Node<B, E, I, K, T, Ts>) -> bool {
        self.node_id == other.node_id
    }
}

impl<B, E, I, K, T, Ts> Eq for Node<B, E, I, K, T, Ts>
where
    B: Bucket<Ts>,
    E: Entity<B, T, Ts>,
    I: Identifier,
    K: Kind,
    T: Tier,
    Ts: TimeStamp,
{
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::bucket::tests::{MyBucket, Ts};
    use crate::entity::tests::{make_device, DeviceType, Level, Nid, TDevice};
    use crate::node::Node;

    pub(crate) type MyNode = Node<MyBucket, TDevice, Nid, DeviceType, Level, Ts>;

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
