use super::bucket::Bucket;
use super::bucket::TimeStamp;
use crate::scheduler::Scheduler;
use krabmaga::engine::{schedule::Schedule, state::State};
use std::any::Any;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct GEngine<B, S, T>
where
    B: Bucket<T>,
    S: Scheduler<T>,
    T: TimeStamp,
{
    end_step: T,
    streaming_interval: T,
    pub bucket: B,
    pub scheduler: S,
    #[builder(default)]
    streaming_step: T,
    #[builder(default)]
    step: T,
}

impl<B, S, T> State for GEngine<B, S, T>
where
    B: Bucket<T>,
    S: Scheduler<T>,
    T: TimeStamp,
{
    fn init(&mut self, schedule: &mut Schedule) {
        self.scheduler.init(schedule);
        self.streaming_step = self.streaming_interval;
        let step = T::from(schedule.step);
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
        self.step = T::from(step);
        self.bucket.update(self.step);
    }

    fn before_step(&mut self, schedule: &mut Schedule) {
        self.scheduler.add_to_schedule(schedule);
        self.bucket.before_uplink();
        if self.step == self.streaming_step {
            self.bucket.streaming_step(self.step);
            self.streaming_step += self.streaming_interval;
        }
    }

    fn after_step(&mut self, schedule: &mut Schedule) {
        self.bucket.after_downlink();
        self.scheduler.remove_from_schedule(schedule);
    }

    fn end_condition(&mut self, _schedule: &mut Schedule) -> bool {
        self.step == self.end_step
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::GEngine;
    use crate::bucket::tests::{MyBucket, Ts};
    use crate::scheduler::tests::{make_scheduler_with_2_devices, MyScheduler};
    use krabmaga::simulate;

    fn make_engine(end_step: Ts, stream_step: Ts) -> GEngine<MyBucket, MyScheduler, Ts> {
        let bucket = MyBucket::new();
        let scheduler = make_scheduler_with_2_devices();
        GEngine::builder()
            .end_step(end_step)
            .streaming_interval(stream_step)
            .bucket(bucket)
            .scheduler(scheduler)
            .build()
    }

    #[test]
    fn test_engine_making() {
        let end_step = Ts::from(100);
        let stream_step = Ts::from(10);
        let engine = make_engine(end_step, stream_step);
        assert_eq!(engine.step, Ts::default());
        assert_eq!(engine.streaming_step, Ts::default());
        assert_eq!(engine.streaming_interval, Ts::from(10));
        assert_eq!(engine.end_step, Ts::from(100));
    }

    #[test]
    fn test_simulation() {
        let end_step = Ts::from(100);
        let stream_step = Ts::from(10);
        let engine = make_engine(end_step, stream_step);
        simulate!(engine, end_step.into(), 1);
    }
}
