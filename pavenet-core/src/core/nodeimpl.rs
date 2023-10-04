use crate::core::core::Core;
use crate::node::node::Node;
use crate::node::power::{PowerSchedule, PowerState};
use downcast_rs::impl_downcast;
use krabmaga::engine::agent::Agent;
use krabmaga::engine::state::State;
use pavenet_config::types::ids::node::NodeId;

#[derive(Clone)]
pub struct NodeImpl {
    node_id: NodeId,
    pub(crate) power_schedule: PowerSchedule,
    pub(crate) node_impl: Box<dyn Node>,
}

impl NodeImpl {
    pub fn new(node_id: NodeId, power_schedule: PowerSchedule, node_impl: Box<dyn Node>) -> Self {
        Self {
            node_id,
            node_impl,
            power_schedule,
        }
    }

    fn power_off(&mut self, core: &mut Core) {
        self.node_impl.set_power_state(PowerState::Off);
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
        self.node_impl.set_power_state(PowerState::On);
        let core_state = state.as_any_mut().downcast_mut::<Core>().unwrap();
        self.node_impl.step(core_state);
        if core_state.step == self.power_schedule.peek_time_to_off() {
            self.power_off(core_state);
        }
    }

    fn after_step(&mut self, state: &mut dyn State) {
        let core_state = state.as_any_mut().downcast_mut::<Core>().unwrap();
        self.node_impl.after_step(core_state);
    }

    fn is_stopped(&self, _state: &mut dyn State) -> bool {
        self.node_impl.power_state() == PowerState::Off
    }
}

dyn_clone::clone_trait_object!(Node);
impl_downcast!(Node);
