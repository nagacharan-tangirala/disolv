use crate::core::nodes::Nodes;
use crate::node::group::NodeGroup;
use hashbrown::HashMap;
use krabmaga::engine::{schedule::Schedule, state::State};
use pavenet_config::config::base::BaseConfig;
use pavenet_config::config::dynamic::DynamicConfig;
use pavenet_config::types::ts::TimeStamp;
use std::any::{Any, TypeId};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct Core {
    pub base_config: BaseConfig,
    pub dyn_config: DynamicConfig,
    pub step: TimeStamp,
    pub nodes: Nodes,
    pub node_collections: HashMap<TypeId, Box<dyn NodeGroup>>,
}

impl State for Core {
    fn init(&mut self, schedule: &mut Schedule) {
        self.node_collections
            .values_mut()
            .for_each(|c| c.init(schedule));
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_state_mut(&mut self) -> &mut dyn State {
        self
    }

    fn as_state(&self) -> &dyn State {
        self
    }

    fn reset(&mut self) {}

    fn update(&mut self, step: u64) {
        self.step = TimeStamp::from(step);
        self.node_collections
            .iter_mut()
            .for_each(|c| c.update(self.step));
    }

    fn before_step(&mut self, schedule: &mut Schedule) {
        self.nodes.power_on(schedule);
        self.node_collections
            .iter_mut()
            .for_each(|c| c.before_step(self.step));

        if self.step > TimeStamp::default()
            && self.step % self.base_config.simulation_settings.sim_streaming_step == 0
        {
            self.node_collections
                .iter_mut()
                .for_each(|c| c.streaming_step(self.step));
        }
    }

    fn after_step(&mut self, schedule: &mut Schedule) {
        self.nodes.power_off(schedule);
        self.node_collections
            .iter_mut()
            .for_each(|c| c.after_step(schedule));
    }

    fn end_condition(&mut self, _schedule: &mut Schedule) -> bool {
        self.step == TimeStamp::from(self.base_config.simulation_settings.sim_duration)
    }
}
