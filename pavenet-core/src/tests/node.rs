use crate::core::core::Core;
use crate::node::info::NodeInfo;
use crate::node::node::Node;
use crate::node::power::PowerState;
use crate::node::receive::Recipient;
use crate::node::transmit::{Payload, Transmitter};

#[derive(Clone, Copy, Debug)]
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

impl Transmitter for TestNode {
    fn transmit(&mut self, _payload: Box<dyn Payload>) {
        todo!()
    }
}

impl Recipient for TestNode {
    fn receive(&mut self, _payloads: &mut Vec<Box<dyn Payload>>) {
        todo!()
    }
}

impl Node for TestNode {
    fn power_state(&self) -> PowerState {
        self.power_state
    }

    fn node_info(&self) -> NodeInfo {
        self.node_info
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
