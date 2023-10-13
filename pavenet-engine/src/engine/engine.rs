use crate::engine::node_set::NodeSet;
use crate::engine::pool_set::PoolSet;
use crate::node::node::Node;
use crate::node::pool::NodePool;
use krabmaga::engine::{schedule::Schedule, state::State};
use krabmaga::log;
use pavenet_core::types::TimeStamp;
use std::any::Any;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct Engine<T, U>
where
    T: Node,
    U: NodePool,
{
    pub step: TimeStamp,
    streaming_step: TimeStamp,
    end_step: TimeStamp,
    pub(crate) node_set: NodeSet<T, U>,
    pub pool_set: PoolSet<U>,
}

impl<T, U> State for Engine<T, U>
where
    T: Node,
    U: NodePool,
{
    fn init(&mut self, schedule: &mut Schedule) {
        let step_str = format!("{}", schedule.step);
        log!(
            LogType::Info,
            String::from("Engine::init step: ") + &step_str
        );
        self.node_set.init();
        self.pool_set.init(schedule);
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
        let step_str = format!("{}", step);
        log!(
            LogType::Info,
            String::from("Engine::update step: ") + &step_str
        );
        self.step = TimeStamp::from(step);
        self.pool_set.update(self.step);
    }

    fn before_step(&mut self, schedule: &mut Schedule) {
        self.node_set.power_on(schedule);
        self.pool_set.before_step(self.step);

        if self.step > TimeStamp::default()
            && self.step.as_u64() % self.streaming_step.as_u64() == 0
        {
            self.pool_set.streaming_step(self.step);
        }
    }

    fn after_step(&mut self, schedule: &mut Schedule) {
        self.node_set.power_off(schedule);
        self.pool_set.after_step(schedule);
    }

    fn end_condition(&mut self, _schedule: &mut Schedule) -> bool {
        self.step == self.end_step
    }
}

#[cfg(test)]
mod tests {
    pub(crate) mod test_engine {
        use crate::engine::engine::Engine;
        use crate::engine::nodeimpl::tests::test_node::TestNode;
        use crate::engine::nodeimpl::tests::test_pool::TestPool;
        use crate::engine::nodeimpl::tests::{make_test_node_set, make_test_pool};
        use crate::engine::pool_set::tests::pool_set::make_test_pool_set;
        use pavenet_core::types::TimeStamp;

        pub(crate) fn make_test_engine() -> Engine<TestNode, TestPool> {
            let test_pool = make_test_pool();
            let test_pool_set = make_test_pool_set(&test_pool);
            let test_node_set = make_test_node_set();
            Engine {
                step: TimeStamp::default(),
                streaming_step: TimeStamp::from(5i64),
                end_step: TimeStamp::from(10i64),
                node_set: test_node_set,
                pool_set: test_pool_set,
            }
        }
    }

    use crate::engine::engine::tests::test_engine::make_test_engine;
    use krabmaga::simulate;

    #[test]
    fn test_engine() {
        let mut engine = make_test_engine();
        simulate!(engine, 10, 1);
        println!("Engine::test_engine");
    }
}
