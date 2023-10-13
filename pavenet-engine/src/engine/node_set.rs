use crate::engine::nodeimpl::NodeImpl;
use crate::node::node::Node;
use crate::node::pool::NodePool;
use crate::node::power::PowerState;
use hashbrown::HashMap;
use krabmaga::engine::schedule::Schedule;
use krabmaga::log;
use pavenet_core::types::NodeId;

#[derive(Default, Clone)]
pub struct NodeSet<T, U>
where
    T: Node,
    U: NodePool,
{
    pub(crate) nodes_by_id: HashMap<NodeId, NodeImpl<T, U>>,
    pub to_add: Vec<NodeId>,
    pub to_pop: Vec<NodeId>,
}

impl<T, U> NodeSet<T, U>
where
    T: Node,
    U: NodePool,
{
    pub fn new(nodes: HashMap<NodeId, NodeImpl<T, U>>) -> Self {
        Self {
            nodes_by_id: nodes,
            ..Default::default()
        }
    }

    pub(crate) fn init(&mut self) {
        log!(LogType::Info, String::from("NodeSet::init"));
        self.to_add.extend(self.nodes_by_id.keys());
    }

    pub(crate) fn power_on(&mut self, schedule: &mut Schedule) {
        for node_id in self.to_add.iter() {
            match self.nodes_by_id.get_mut(node_id) {
                Some(node) => {
                    schedule.schedule_repeating(
                        Box::new(node.clone()),
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
                    schedule.dequeue(Box::new(node.clone()), (*node_id).into());
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

    #[test]
    fn add_to_schedule() {
        let mut nodes = make_test_node_set();
        let mut schedule = Schedule::new();
        nodes.to_add.push(1.into());
        nodes.power_on(&mut schedule);
        assert_eq!(schedule.events.len(), 1);
    }

    #[test]
    fn pop_from_schedule() {
        let mut nodes = make_test_node_set();
        let mut schedule = Schedule::new();
        nodes.to_pop.push(1.into());
        nodes.power_off(&mut schedule);
        assert_eq!(schedule.events.len(), 0);
    }

    #[test]
    fn power_off() {
        let mut nodes = make_test_node_set();
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
        let mut nodes = make_test_node_set();
        nodes.init();
        assert_eq!(nodes.to_add.len(), 10);
    }
}
