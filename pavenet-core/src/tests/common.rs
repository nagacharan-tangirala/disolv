use crate::core::nodeimpl::NodeImpl;
use crate::core::nodes::Nodes;
use crate::node::info::NodeInfo;
use crate::node::node::Node;
use crate::node::power::{PowerSchedule, SCHEDULE_SIZE};
use crate::tests::node::TestNode;
use hashbrown::HashMap;
use pavenet_config::config::base::NodeType;
use pavenet_config::types::ids::node::NodeId;
use pavenet_config::types::order::Order;
use pavenet_config::types::ts::TimeStamp;

pub(crate) fn make_node_impls() -> Vec<NodeImpl> {
    let mut nodes: Vec<NodeImpl> = Vec::with_capacity(10);
    for i in 0..10 {
        let node_info = NodeInfo::builder()
            .node_type(NodeType::Vehicle)
            .node_class(1)
            .id(i.into())
            .order(Order::from(1))
            .build();
        let test_node = TestNode::new(node_info);
        let power_schedule = make_power_schedule();
        let test_node_impl = NodeImpl::new(i.into(), power_schedule, Box::new(test_node));
        nodes.push(test_node_impl);
    }
    nodes
}

pub(crate) fn make_dyn_nodes() -> Vec<Box<dyn Node>> {
    let mut nodes: Vec<Box<dyn Node>> = Vec::with_capacity(10);
    for i in 0..10 {
        let node_info = NodeInfo::builder()
            .node_type(NodeType::Vehicle)
            .node_class(1)
            .id(i.into())
            .order(Order::from(1))
            .build();
        let test_node = TestNode::new(node_info);
        nodes.push(Box::new(test_node));
    }
    nodes
}

fn make_power_schedule() -> PowerSchedule {
    let mut on_times: [Option<TimeStamp>; SCHEDULE_SIZE] = [None; SCHEDULE_SIZE];
    let mut off_times: [Option<TimeStamp>; SCHEDULE_SIZE] = [None; SCHEDULE_SIZE];
    for i in 0..SCHEDULE_SIZE {
        on_times[i] = Some(TimeStamp::from(0u64));
        off_times[i] = Some(TimeStamp::from(10u64));
    }
    PowerSchedule::new(on_times, off_times)
}

pub(crate) fn make_node_group() -> Nodes {
    let nodes = make_dyn_nodes();
    let mut power_schedule_map: HashMap<NodeId, PowerSchedule> = HashMap::new();
    for i in 0..10 {
        let power_schedule = make_power_schedule();
        power_schedule_map.insert(i.into(), power_schedule);
    }
    Nodes::new(nodes, power_schedule_map)
}
