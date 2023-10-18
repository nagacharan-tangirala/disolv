use super::bucket::Bucket;
use super::bucket::TimeStamp;
use krabmaga::engine::{schedule::Schedule, state::State};
use std::any::Any;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct Engine<B, S>
where
    B: Bucket<S>,
    S: TimeStamp,
{
    end_step: S,
    streaming_interval: S,
    pub bucket: B, // Hashmap might be expensive, as list is potentially tiny
    #[builder(default)]
    streaming_step: S,
    #[builder(default)]
    pub step: S,
}

impl<B, S> State for Engine<B, S>
where
    B: Bucket<S>,
    S: TimeStamp,
{
    fn init(&mut self, schedule: &mut Schedule) {
        self.streaming_step = self.streaming_interval;
        self.bucket.init(schedule);
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
        self.step = S::from(step);
        self.bucket.update(self.step);
    }

    fn before_step(&mut self, schedule: &mut Schedule) {
        self.bucket.before_step(schedule);
        if self.step == self.streaming_step {
            self.bucket.streaming_step(self.step);
            self.streaming_step += self.streaming_interval;
        }
    }

    fn after_step(&mut self, schedule: &mut Schedule) {
        self.bucket.after_step(schedule);
    }

    fn end_condition(&mut self, _schedule: &mut Schedule) -> bool {
        self.step == self.end_step
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::Engine;
    use crate::bucket::tests::{MyBucket, Ts};
    use crate::entity::tests::{make_device, DeviceType, Nid};
    use crate::node::tests::as_node;
    use krabmaga::simulate;

    fn make_bucket_a(order: i32) -> MyBucket {
        let device_a = make_device(Nid::from(1), DeviceType::TypeA, order);
        let node_a = as_node(device_a);
        let mut bucket_a = MyBucket::new(DeviceType::TypeA);
        bucket_a.add(node_a);
        bucket_a
    }

    fn make_engine_with_type_a_bucket(
        end_step: Ts,
        stream_step: Ts,
    ) -> Engine<DeviceType, MyBucket, Ts> {
        let bucket_a = make_bucket_a(1);
        Engine::builder()
            .end_step(end_step)
            .streaming_interval(stream_step)
            .buckets_by_kind(vec![(DeviceType::TypeA, bucket_a)])
            .build()
    }

    fn make_bucket_b(order: i32) -> MyBucket {
        let device_b = make_device(Nid::from(2), DeviceType::TypeB, order);
        let node_b = as_node(device_b);
        let mut bucket_b = MyBucket::new(DeviceType::TypeB);
        bucket_b.add(node_b);
        bucket_b
    }

    fn make_2_buckets() -> Vec<(DeviceType, MyBucket)> {
        let bucket_a = make_bucket_a(1);
        let bucket_b = make_bucket_b(2);
        let mut buckets: Vec<(DeviceType, MyBucket)> = Vec::new();
        buckets.push((DeviceType::TypeA, bucket_a));
        buckets.push((DeviceType::TypeB, bucket_b));
        buckets
    }

    fn make_engine_with_2_buckets(
        end_step: Ts,
        stream_step: Ts,
    ) -> Engine<DeviceType, MyBucket, Ts> {
        let buckets = make_2_buckets();
        Engine::builder()
            .end_step(end_step)
            .streaming_interval(stream_step)
            .buckets_by_kind(buckets)
            .build()
    }

    #[test]
    fn test_engine_making() {
        let end_step = Ts::from(100);
        let stream_step = Ts::from(10);
        let engine = make_engine_with_2_buckets(end_step, stream_step);
        assert_eq!(engine.buckets_by_kind.len(), 2);
        assert_eq!(engine.step, Ts::default());
        assert_eq!(engine.streaming_step, Ts::default());
        assert_eq!(engine.streaming_interval, Ts::from(10));
        assert_eq!(engine.end_step, Ts::from(100));
    }

    #[test]
    fn test_bucket_add() {
        let end_step = Ts::from(100);
        let stream_step = Ts::from(10);
        let mut engine = make_engine_with_type_a_bucket(end_step, stream_step);
        assert_eq!(engine.buckets_by_kind.len(), 1);
        let bucket_b = make_bucket_b(2);
        engine.add(DeviceType::TypeB, bucket_b);
        assert_eq!(engine.buckets_by_kind.len(), 2);
    }

    #[test]
    fn test_bucket_of() {
        let end_step = Ts::from(100);
        let stream_step = Ts::from(10);
        let mut engine = make_engine_with_2_buckets(end_step, stream_step);
        let bucket_a = engine.bucket_of(DeviceType::TypeA);
        assert!(bucket_a.is_some());
        let bucket_b = engine.bucket_of(DeviceType::TypeB);
        assert!(bucket_b.is_some());
    }

    #[test]
    fn test_simulation() {
        let end_step = Ts::from(100);
        let stream_step = Ts::from(10);
        let mut engine = make_engine_with_type_a_bucket(end_step, stream_step);
        let bucket_b = make_bucket_b(2);
        engine.add(DeviceType::TypeB, bucket_b);
        simulate!(engine, end_step.into(), 1);
    }
}
