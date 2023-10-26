use crate::bucket::{Bucket, Scheduler, TimeStamp};
use crate::entity::{Entity, Identifier, Kind, Tier};
use crate::node::Node;
use krabmaga::engine::schedule::Schedule;
use std::collections::HashMap;

#[derive(Default, Clone)]
pub(crate) struct NodeScheduler<B, E, I, K, T, Ts>
where
    B: Bucket<Ts>,
    E: Entity<B, T, Ts>,
    I: Identifier,
    K: Kind,
    T: Tier,
    Ts: TimeStamp,
{
    nodes: HashMap<I, Node<B, E, I, K, T, Ts>>,
    to_pop: Vec<I>,
    to_add: Vec<I>,
}

impl<B, E, I, K, T, Ts> NodeScheduler<B, E, I, K, T, Ts>
where
    B: Bucket<Ts>,
    E: Entity<B, T, Ts>,
    I: Identifier,
    K: Kind,
    T: Tier,
    Ts: TimeStamp,
{
    pub fn new(entities: HashMap<I, Node<B, E, I, K, T, Ts>>) -> Self {
        Self {
            nodes: entities,
            to_pop: Vec::new(),
            to_add: Vec::new(),
        }
    }

    pub fn pop(&mut self, node_id: I) {
        self.to_pop.push(node_id);
    }

    pub fn add(&mut self, node_id: I) {
        self.to_add.push(node_id);
    }
}

impl<B, E, I, K, T, Ts> Scheduler<Ts> for NodeScheduler<B, E, I, K, T, Ts>
where
    B: Bucket<Ts>,
    E: Entity<B, T, Ts>,
    I: Identifier,
    K: Kind,
    T: Tier,
    Ts: TimeStamp,
{
    fn init(&mut self, schedule: &mut Schedule) {
        for (id, node) in self.nodes.iter_mut() {
            schedule.schedule_repeating(
                Box::new(node.clone()),
                id.as_u32(),
                node.entity.time_to_add().as_f32(),
                node.entity.tier().as_i32(),
            );
        }
    }

    fn add_to_schedule(&mut self, schedule: &mut Schedule) {
        for id in self.to_add.iter() {
            match self.nodes.get_mut(id) {
                Some(node) => {
                    schedule.schedule_repeating(
                        Box::new(node.clone()),
                        id.as_u32(),
                        node.entity.time_to_add().as_f32(),
                        node.entity.tier().as_i32(),
                    );
                }
                None => panic!("Could not find node {}", id),
            }
        }
    }

    fn remove_from_schedule(&mut self, schedule: &mut Schedule) {
        for id in self.to_pop.iter() {
            match self.nodes.get_mut(id) {
                Some(node) => {
                    node.entity.stop();
                    schedule.dequeue(Box::new(node.clone()), id.as_u32());
                }
                None => panic!("Could not find node {}", id),
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::bucket::tests::{MyBucket, Ts};
    use crate::entity::tests::{make_device, DeviceType, Level, Nid, TDevice};
    use crate::node::tests::as_node;

    pub(crate) type MyScheduler = NodeScheduler<MyBucket, TDevice, Nid, DeviceType, Level, Ts>;

    pub(crate) fn make_scheduler_with_2_devices() -> MyScheduler {
        let mut nodes = HashMap::new();
        let device_a = make_device(Nid::from(1), DeviceType::TypeA, 1);
        let device_b = make_device(Nid::from(2), DeviceType::TypeB, 1);
        let node_a = as_node(device_a);
        let node_b = as_node(device_b);
        nodes.insert(node_a.node_id, node_a);
        nodes.insert(node_b.node_id, node_b);
        MyScheduler::new(nodes)
    }

    #[test]
    fn test_scheduler_init() {
        let mut scheduler = make_scheduler_with_2_devices();
        let mut schedule = Schedule::new();
        scheduler.init(&mut schedule);
        assert_eq!(schedule.events.len(), 2);
    }
}
