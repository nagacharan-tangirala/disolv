use super::bucket::{Bucket, TimeMS};
use crate::scheduler::Scheduler;
use krabmaga::engine::{schedule::Schedule, state::State};
use std::any::Any;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct GEngine<B>
where
    B: Bucket,
{
    end_step: TimeMS,
    streaming_interval: TimeMS,
    pub bucket: B,
    step_size: TimeMS,
    #[builder(default)]
    streaming_step: TimeMS,
    #[builder(default)]
    step: TimeMS,
}

impl<B> State for GEngine<B>
where
    B: Bucket,
{
    fn init(&mut self, schedule: &mut Schedule) {
        self.bucket.scheduler().init(schedule);
        self.streaming_step = self.streaming_interval;
        let step = TimeMS::from(schedule.step);
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
        self.step = TimeMS::from(step) * self.step_size;
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
        self.bucket.scheduler().clear_lists();
    }

    fn end_condition(&mut self, schedule: &mut Schedule) -> bool {
        if self.step == self.end_step {
            self.bucket.end(TimeMS::from(schedule.step));
            return true;
        }
        false
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::GEngine;
    use crate::bucket::tests::MyBucket;
    use crate::bucket::TimeMS;
    use crate::entity::tests::{make_device, DeviceType, Level, TDevice};
    use crate::node::{GNode, NodeId};
    use crate::scheduler::tests::make_scheduler_with_2_devices;
    use krabmaga::simulate;

    pub(crate) type MyNode = GNode<MyBucket, TDevice, Level>;

    pub(crate) fn as_node(device: TDevice) -> MyNode {
        GNode::new(device.id, device)
    }

    fn make_engine(end_step: TimeMS, stream_step: TimeMS, step_size: TimeMS) -> GEngine<MyBucket> {
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
        let end_step = TimeMS::from(100);
        let stream_step = TimeMS::from(10);
        let step_size = TimeMS::from(1);
        let engine = make_engine(end_step, stream_step, step_size);
        assert_eq!(engine.step, TimeMS::default());
        assert_eq!(engine.streaming_step, TimeMS::default());
        assert_eq!(engine.streaming_interval, TimeMS::from(10));
        assert_eq!(engine.end_step, TimeMS::from(100));
    }

    #[test]
    fn test_delayed_schedule() {
        // Make sure second device is scheduled with a delay
        let end_step = TimeMS::from(100);
        let stream_step = TimeMS::from(10);
        let step_size = TimeMS::from(1);
        let mut engine = make_engine(end_step, stream_step, step_size);
        let node_2 = as_node(make_device(NodeId::from(2), DeviceType::TypeB, 2));
        engine.bucket.scheduler.add(node_2.node_id);
        engine.bucket.scheduler.add(NodeId::from(2));
    }

    #[test]
    fn test_simulation() {
        let end_step = TimeMS::from(100);
        let stream_step = TimeMS::from(50);
        let step_size = TimeMS::from(10);
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
