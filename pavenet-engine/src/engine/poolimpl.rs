use crate::engine::nodeimpl::NodeImpl;
use crate::node::node::Node;
use crate::node::power::{PowerSchedule, PowerState};
use hashbrown::HashMap;
use krabmaga::engine::schedule::Schedule;
use pavenet_core::named::ids::node::NodeId;

#[derive(Default)]
pub struct PoolImpl {
    nodes_by_id: HashMap<NodeId, NodeImpl>,
    pub to_add: Vec<NodeId>,
    pub to_pop: Vec<NodeId>,
}

impl PoolImpl {
    pub fn new(
        dyn_nodes: HashMap<NodeId, Box<dyn Node>>,
        power_schedule_map: HashMap<NodeId, PowerSchedule>,
    ) -> Self {
        let mut by_id: HashMap<NodeId, NodeImpl> = HashMap::new();
        for (node_id, dyn_node) in dyn_nodes.into_iter() {
            let node = NodeImpl::new(node_id, power_schedule_map[&node_id], dyn_node);
            by_id.insert(node_id, node);
        }
        Self {
            nodes_by_id: by_id,
            ..Default::default()
        }
    }

    pub(crate) fn init(&mut self) {
        self.to_add.extend(self.nodes_by_id.keys());
    }

    pub(crate) fn power_on(&mut self, schedule: &mut Schedule) {
        for node_id in self.to_add.iter() {
            match self.nodes_by_id.get_mut(node_id) {
                Some(node) => {
                    schedule.schedule_repeating(
                        node.as_agent(),
                        node.node_id.as_u32(),
                        node.power_schedule.pop_time_to_on().as_f32(),
                        node.node.node_order(),
                    );
                }
                None => panic!("Could not find node {}", node_id),
            }
        }
    }

    pub(crate) fn power_off(&mut self, schedule: &mut Schedule) {
        for node_id in self.to_pop.iter() {
            match self.nodes_by_id.get_mut(node_id) {
                Some(node) => {
                    node.node.set_power_state(PowerState::Off);
                    schedule.dequeue(node.clone().as_agent(), (*node_id).into());
                }
                None => panic!("Could not find node {}", node_id),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::nodeimpl::tests::*;

    pub(crate) fn make_node_pool() -> PoolImpl {
        let nodes = make_dyn_nodes();
        let mut power_schedule_map: HashMap<NodeId, PowerSchedule> = HashMap::new();
        for i in 0..10 {
            let power_schedule = make_power_schedule();
            power_schedule_map.insert(i.into(), power_schedule);
        }
        PoolImpl::new(nodes, power_schedule_map)
    }

    #[test]
    fn add_to_schedule() {
        let mut nodes = make_node_pool();
        let mut schedule = Schedule::new();
        nodes.to_add.push(1.into());
        nodes.power_on(&mut schedule);
        assert_eq!(schedule.events.len(), 1);
    }

    #[test]
    fn pop_from_schedule() {
        let mut nodes = make_node_pool();
        let mut schedule = Schedule::new();
        nodes.to_pop.push(1.into());
        nodes.power_off(&mut schedule);
        assert_eq!(schedule.events.len(), 0);
    }

    #[test]
    fn power_off() {
        let mut nodes = make_node_pool();
        let mut schedule = Schedule::new();
        nodes.to_pop.push(1.into());
        nodes.power_off(&mut schedule);
        let node_id_1 = NodeId::from(1);
        assert_eq!(
            nodes.nodes_by_id[&node_id_1].node.power_state(),
            PowerState::Off
        );
        assert_eq!(schedule.events.len(), 0);
    }

    #[test]
    fn init() {
        let mut nodes = make_node_pool();
        nodes.init();
        assert_eq!(nodes.to_add.len(), 10);
    }
}
