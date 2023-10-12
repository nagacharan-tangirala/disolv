use crate::engine::core::Core;
use crate::node::node::Node;
use crate::node::power::PowerState;
use pavenet_core::structs::NodeInfo;

#[derive(Clone, Debug)]
pub(crate) struct TestNode {
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

impl Node for TestNode {
    fn power_state(&self) -> PowerState {
        self.power_state
    }

    fn node_order(&self) -> i32 {
        self.node_info.order.as_i32()
    }

    fn set_power_state(&mut self, power_state: PowerState) {
        self.power_state = power_state;
    }

    fn step(&mut self, _core: &mut Core) {
        println!("TestNode::step");
    }

    fn after_step(&mut self, _core: &mut Core) {
        println!("TestNode::after_step");
    }
}
