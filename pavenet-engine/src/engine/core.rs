use crate::engine::nodes::PoolImpl;
use crate::node::pool::NodePool;
use krabmaga::engine::{schedule::Schedule, state::State};
use pavenet_core::types::TimeStamp;
use std::any::Any;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct Core {
    pub step: TimeStamp,
    streaming_step: TimeStamp,
    end_step: TimeStamp,
    pub(crate) pool_impl: PoolImpl,
    pub node_pools: Vec<Box<dyn NodePool>>,
}

impl State for Core {
    fn init(&mut self, schedule: &mut Schedule) {
        self.node_pools.iter_mut().for_each(|c| c.init(schedule));
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
        self.node_pools.iter_mut().for_each(|c| c.update(self.step));
    }

    fn before_step(&mut self, schedule: &mut Schedule) {
        self.pool_impl.power_on(schedule);
        self.node_pools
            .iter_mut()
            .for_each(|c| c.before_step(self.step));

        if self.step > TimeStamp::default()
            && self.step.as_u64() % self.streaming_step.as_u64() == 0
        {
            self.node_pools
                .iter_mut()
                .for_each(|c| c.streaming_step(self.step));
        }
    }

    fn after_step(&mut self, schedule: &mut Schedule) {
        self.pool_impl.power_off(schedule);
        self.node_pools
            .iter_mut()
            .for_each(|c| c.after_step(schedule));
    }

    fn end_condition(&mut self, _schedule: &mut Schedule) -> bool {
        self.step == self.end_step
    }
}
