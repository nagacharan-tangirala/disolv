use crate::node::node::Node;
use crate::node::power::PowerState;
use hashbrown::HashMap;
use krabmaga::engine::schedule::Schedule;
use pavenet_config::types::ids::node::NodeId;

pub struct Nodes {
    nodes_by_id: HashMap<NodeId, Box<dyn Node>>,
    pub(crate) to_add: Vec<NodeId>,
    pub(crate) to_pop: Vec<NodeId>,
}

impl Nodes {
    pub fn new(nodes: Vec<Box<dyn Node>>) -> Self {
        let mut by_id: HashMap<NodeId, Box<dyn Node>> = HashMap::new();
        for node in nodes.into_iter() {
            let d_info = node.node_info();
            by_id.insert(d_info.id, node);
        }
        Self {
            nodes_by_id: by_id,
            ..Default::default()
        }
    }

    pub(crate) fn power_on(&mut self, schedule: &mut Schedule) {
        for node_id in self.to_add.into_iter() {
            match self.nodes_by_id.get_mut(&node_id) {
                Some(node) => schedule.schedule_repeating(
                    node.as_agent(),
                    node.node_info().id.into(),
                    node.node_info().power_on_time(),
                    node.node_info().hierarchy.into(),
                ),
                None => {
                    panic!("Could not find node {}", node_id);
                }
            }
        }
    }

    pub(crate) fn power_off(&mut self, schedule: &mut Schedule) {
        for node_id in self.to_pop.into_iter() {
            match self.nodes_by_id.get_mut(&node_id) {
                Some(node) => {
                    node.set_power(PowerState::Off);
                    schedule.dequeue(node.as_agent(), node_id.into());
                }
                None => {
                    panic!("Could not find node {}", node_id);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::info::NodeInfo;
    use crate::node::node::Node;
    use pavenet_config::config::base::NodeType;
    use pavenet_config::types::hierarchy::Hierarchy;

    #[derive(Clone, Default, Copy, Debug, PartialEq)]
    pub struct TestNode {
        pub node_info: NodeInfo,
        pub power_state: PowerState,
    }

    impl TestNode {
        pub fn new(node_info: NodeInfo) -> Self {
            Self {
                node_info,
                power_state: PowerState::Off,
            }
        }
    }

    fn make_test_nodes() -> Vec<Box<dyn Node>> {
        let mut nodes: Vec<Box<dyn Node>> = Vec::new();
        for i in 0..10 {
            let node_info = NodeInfo::builder()
                .node_type(NodeType::Vehicle)
                .node_class(1)
                .id(i.into())
                .hierarchy(Hierarchy::from(1))
                .build();
            let node = TestNode::new(node_info);
            nodes.push(Box::new(node));
        }
        nodes
    }

    #[test]
    fn nodes_new() {
        let nodes = make_test_nodes();
        let nodes = Nodes::new(nodes);
        assert_eq!(nodes.nodes_by_id.len(), 10);
    }

    #[test]
    fn power_on() {
        let nodes = make_test_nodes();
        let mut nodes = Nodes::new(nodes);
        let mut schedule = Schedule::new();
        nodes.to_add.push(1.into());
        nodes.to_add.push(2.into());
        nodes.power_on(&mut schedule);
        assert_eq!(schedule.events.len(), 1);
    }

    #[test]
    fn power_off() {
        let nodes = make_test_nodes();
        let mut nodes = Nodes::new(nodes);
        let mut schedule = Schedule::new();
        nodes.to_pop.push(1.into());
        nodes.power_off(&mut schedule);
        assert_eq!(schedule.events.len(), 0);
    }
}
