use crate::engine::nodeimpl::NodeImpl;
use crate::engine::nodes::PoolImpl;
use crate::node::node::Node;
use crate::node::power::{PowerSchedule, SCHEDULE_SIZE};
use crate::tests::node::TestNode;
use hashbrown::HashMap;
use pavenet_core::enums::NodeType;
use pavenet_core::named::class::Class;
use pavenet_core::structs::NodeInfo;
use pavenet_core::types::{NodeId, Order, TimeStamp};

pub(crate) fn make_node_impls() -> Vec<NodeImpl> {
    let mut nodes: Vec<NodeImpl> = Vec::with_capacity(10);
    for i in 0..10 {
        let node_info = NodeInfo::builder()
            .node_type(NodeType::Vehicle)
            .node_class(Class::from(1))
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

pub(crate) fn make_dyn_nodes() -> HashMap<NodeId, Box<dyn Node>> {
    let mut nodes: HashMap<NodeId, Box<dyn Node>> = HashMap::with_capacity(10);
    for i in 0..10 {
        let node_info = NodeInfo::builder()
            .node_type(NodeType::Vehicle)
            .node_class(Class::from(1))
            .id(i.into())
            .order(Order::from(1))
            .build();
        let test_node = TestNode::new(node_info);
        let nod_dyn: Box<dyn Node> = Box::new(test_node);
        nodes.insert(NodeId::from(i), nod_dyn);
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

pub(crate) fn make_node_pool() -> PoolImpl {
    let nodes = make_dyn_nodes();
    let mut power_schedule_map: HashMap<NodeId, PowerSchedule> = HashMap::new();
    for i in 0..10 {
        let power_schedule = make_power_schedule();
        power_schedule_map.insert(i.into(), power_schedule);
    }
    PoolImpl::new(nodes, power_schedule_map)
}
