use crate::engine::core::Core;
use crate::node::node::Node;
use crate::node::power::{PowerSchedule, PowerState};
use downcast_rs::impl_downcast;
use krabmaga::engine::agent::Agent;
use krabmaga::engine::state::State;
use pavenet_core::named::ids::node::NodeId;
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Clone)]
pub struct NodeImpl {
    pub(crate) node_id: NodeId,
    pub(crate) power_schedule: PowerSchedule,
    pub(crate) node: Box<dyn Node>,
}

impl NodeImpl {
    pub fn new(node_id: NodeId, power_schedule: PowerSchedule, dyn_node: Box<dyn Node>) -> Self {
        Self {
            node_id,
            node: dyn_node,
            power_schedule,
        }
    }

    fn power_off(&mut self, core: &mut Core) {
        self.node.set_power_state(PowerState::Off);
        self.power_schedule.pop_time_to_off();
        core.nodes.to_pop.push(self.node_id);

        let time_stamp = self.power_schedule.pop_time_to_on();
        if time_stamp > core.step {
            core.nodes.to_add.push(self.node_id);
        }
    }

    pub fn as_agent(&self) -> Box<dyn Agent> {
        Box::new(self.clone())
    }
}

impl Agent for NodeImpl {
    fn step(&mut self, state: &mut dyn State) {
        self.node.set_power_state(PowerState::On);
        let core_state = state.as_any_mut().downcast_mut::<Core>().unwrap();
        self.node.step(core_state);
        if core_state.step == self.power_schedule.peek_time_to_off() {
            self.power_off(core_state);
        }
    }

    fn after_step(&mut self, state: &mut dyn State) {
        let core_state = state.as_any_mut().downcast_mut::<Core>().unwrap();
        self.node.after_step(core_state);
    }

    fn is_stopped(&self, _state: &mut dyn State) -> bool {
        self.node.power_state() == PowerState::Off
    }
}

impl Hash for NodeImpl {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.node_id.hash(state);
    }
}

impl fmt::Display for NodeImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.node_id)
    }
}

impl Eq for NodeImpl {}

impl PartialEq for NodeImpl {
    fn eq(&self, other: &NodeImpl) -> bool {
        self.node_id == other.node_id
    }
}

dyn_clone::clone_trait_object!(Node);
impl_downcast!(Node);
