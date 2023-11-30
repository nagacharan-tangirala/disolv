use crate::bucket::Bucket;
use crate::entity::{Entity, Tier};
use crate::node::{GNode, NodeId};
use hashbrown::HashMap;
use krabmaga::engine::schedule::Schedule;

/// A trait used to represent a scheduler. A scheduler is used to schedule entities. The order
/// of calling the scheduler's functions is important to ensure the correct behavior of the engine.
/// Adding and removing entities should be handled in this trait.
pub trait Scheduler: Clone + Send + Sync + 'static {
    fn init(&mut self, schedule: &mut Schedule);
    fn add_to_schedule(&mut self, schedule: &mut Schedule);
    fn remove_from_schedule(&mut self, schedule: &mut Schedule);
}

/// A struct that represents a scheduler for nodes. This is used to schedule nodes when they are
/// added or removed from the network.
#[derive(Default, Clone)]
pub struct GNodeScheduler<B, E, T>
where
    B: Bucket,
    E: Entity<B, T>,
    T: Tier,
{
    nodes: HashMap<NodeId, GNode<B, E, T>>,
    to_pop: Vec<NodeId>,
    to_add: Vec<NodeId>,
}

impl<B, E, T> GNodeScheduler<B, E, T>
where
    B: Bucket,
    E: Entity<B, T>,
    T: Tier,
{
    pub fn new(entities: HashMap<NodeId, GNode<B, E, T>>) -> Self {
        Self {
            nodes: entities,
            to_pop: Vec::new(),
            to_add: Vec::new(),
        }
    }

    pub fn pop(&mut self, node_id: NodeId) {
        self.to_pop.push(node_id);
    }

    pub fn add(&mut self, node_id: NodeId) {
        self.to_add.push(node_id);
    }
}

impl<B, E, T> Scheduler for GNodeScheduler<B, E, T>
where
    B: Bucket,
    E: Entity<B, T>,
    T: Tier,
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
    use crate::bucket::tests::MyBucket;
    use crate::engine::tests::as_node;
    use crate::entity::tests::{make_device, DeviceType, Level, TDevice};

    pub(crate) type MyScheduler = GNodeScheduler<MyBucket, TDevice, Level>;

    pub(crate) fn make_scheduler_with_2_devices() -> MyScheduler {
        let mut nodes = HashMap::new();
        let device_a = make_device(NodeId::from(1), DeviceType::TypeA, 1);
        let device_b = make_device(NodeId::from(2), DeviceType::TypeB, 2);
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
