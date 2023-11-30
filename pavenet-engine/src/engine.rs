use super::bucket::{Bucket, TimeS};
use crate::scheduler::Scheduler;
use krabmaga::engine::{schedule::Schedule, state::State};
use std::any::Any;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct GEngine<B>
where
    B: Bucket,
{
    end_step: TimeS,
    streaming_interval: TimeS,
    pub bucket: B,
    step_size: TimeS,
    #[builder(default)]
    streaming_step: TimeS,
    #[builder(default)]
    step: TimeS,
}

impl<B> State for GEngine<B>
where
    B: Bucket,
{
    fn init(&mut self, schedule: &mut Schedule) {
        self.bucket.scheduler().init(schedule);
        self.streaming_step = self.streaming_interval;
        let step = TimeS::from(schedule.step);
        self.bucket.init(step);
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
        self.step = TimeS::from(step) * self.step_size;
        self.bucket.update(self.step);
    }

    fn before_step(&mut self, schedule: &mut Schedule) {
        self.bucket.scheduler().add_to_schedule(schedule);
        self.bucket.before_uplink();
        if self.step == self.streaming_step {
            self.bucket.streaming_step(self.step);
            self.streaming_step += self.streaming_interval;
        }
    }

    fn after_step(&mut self, schedule: &mut Schedule) {
        self.bucket.after_downlink();
        self.bucket.scheduler().remove_from_schedule(schedule);
    }

    fn end_condition(&mut self, _schedule: &mut Schedule) -> bool {
        self.step == self.end_step
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::GEngine;
    use crate::bucket::tests::MyBucket;
    use crate::bucket::TimeS;
    use crate::entity::tests::{make_device, DeviceType, Level, TDevice};
    use crate::node::{GNode, NodeId};
    use crate::scheduler::tests::make_scheduler_with_2_devices;
    use krabmaga::simulate;

    pub(crate) type MyNode = GNode<MyBucket, TDevice, Level>;

    pub(crate) fn as_node(device: TDevice) -> MyNode {
        GNode::new(device.id, device)
    }

    fn make_engine(end_step: TimeS, stream_step: TimeS, step_size: TimeS) -> GEngine<MyBucket> {
        let bucket = MyBucket::new();
        let scheduler = make_scheduler_with_2_devices();
        GEngine::builder()
            .end_step(end_step)
            .streaming_interval(stream_step)
            .bucket(bucket)
            .step_size(step_size)
            .build()
    }

    #[test]
    fn test_engine_making() {
        let end_step = TimeS::from(100);
        let stream_step = TimeS::from(10);
        let step_size = TimeS::from(1);
        let engine = make_engine(end_step, stream_step, step_size);
        assert_eq!(engine.step, TimeS::default());
        assert_eq!(engine.streaming_step, TimeS::default());
        assert_eq!(engine.streaming_interval, TimeS::from(10));
        assert_eq!(engine.end_step, TimeS::from(100));
    }

    #[test]
    fn test_simulation() {
        let end_step = TimeS::from(100);
        let stream_step = TimeS::from(50);
        let step_size = TimeS::from(10);
        let engine = make_engine(end_step, stream_step, step_size);
        simulate!(engine, end_step.as_u64(), 1);
    }

    #[test]
    fn test_make_nodes() {
        let device_a = make_device(NodeId::from(1), DeviceType::TypeA, 1);
        let node_a = as_node(device_a);
        assert_eq!(node_a.node_id, NodeId::from(1));
    }
}
