use super::bucket::Bucket;
use super::bucket::TimeStamp;
use crate::entity::{Entity, Identifier, Kind, Tier};
use crate::scheduler::GNodeScheduler;
use crate::scheduler::Scheduler;
use krabmaga::engine::agent::Agent;
use krabmaga::engine::{schedule::Schedule, state::State};
use std::any::Any;
use std::fmt;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use typed_builder::TypedBuilder;

#[derive(Clone, Default)]
pub struct GNode<B, E, I, K, T, Ts>
where
    B: Bucket<Ts>,
    E: Entity<B, T, Ts>,
    I: Identifier,
    K: Kind,
    T: Tier,
    Ts: TimeStamp,
{
    pub node_id: I,
    pub entity: E,
    pub kind: K,
    _marker: std::marker::PhantomData<fn() -> (B, T, Ts)>,
}

impl<B, E, I, K, T, Ts> Agent for GNode<B, E, I, K, T, Ts>
where
    B: Bucket<Ts>,
    E: Entity<B, T, Ts>,
    I: Identifier,
    K: Kind,
    T: Tier,
    Ts: TimeStamp,
{
    fn step(&mut self, state: &mut dyn State) {
        let engine: &mut GEngine<B, GNodeScheduler<B, E, I, K, T, Ts>, Ts> = state
            .as_any_mut()
            .downcast_mut::<GEngine<B, GNodeScheduler<B, E, I, K, T, Ts>, Ts>>()
            .unwrap();
        self.entity.uplink_stage(&mut engine.bucket);
    }

    fn after_step(&mut self, state: &mut dyn State) {
        let engine = state
            .as_any_mut()
            .downcast_mut::<GEngine<B, GNodeScheduler<B, E, I, K, T, Ts>, Ts>>()
            .unwrap();
        self.entity.downlink_stage(&mut engine.bucket);
    }

    fn is_stopped(&self, _state: &mut dyn State) -> bool {
        self.entity.is_stopped()
    }
}

impl<B, E, I, K, T, Ts> GNode<B, E, I, K, T, Ts>
where
    B: Bucket<Ts>,
    E: Entity<B, T, Ts>,
    I: Identifier,
    K: Kind,
    T: Tier,
    Ts: TimeStamp,
{
    pub fn new(node_id: I, node: E, kind: K) -> Self {
        Self {
            node_id,
            entity: node,
            kind,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<B, E, I, K, T, Ts> Hash for GNode<B, E, I, K, T, Ts>
where
    B: Bucket<Ts>,
    E: Entity<B, T, Ts>,
    I: Identifier,
    K: Kind,
    T: Tier,
    Ts: TimeStamp,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.node_id.hash(state);
    }
}

impl<B, E, I, K, T, Ts> Display for GNode<B, E, I, K, T, Ts>
where
    B: Bucket<Ts>,
    E: Entity<B, T, Ts>,
    I: Identifier,
    K: Kind,
    T: Tier,
    Ts: TimeStamp,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.node_id)
    }
}

impl<B, E, I, K, T, Ts> PartialEq for GNode<B, E, I, K, T, Ts>
where
    B: Bucket<Ts>,
    E: Entity<B, T, Ts>,
    I: Identifier,
    K: Kind,
    T: Tier,
    Ts: TimeStamp,
{
    fn eq(&self, other: &GNode<B, E, I, K, T, Ts>) -> bool {
        self.node_id == other.node_id
    }
}

impl<B, E, I, K, T, Ts> Eq for GNode<B, E, I, K, T, Ts>
where
    B: Bucket<Ts>,
    E: Entity<B, T, Ts>,
    I: Identifier,
    K: Kind,
    T: Tier,
    Ts: TimeStamp,
{
}

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
    use super::GNode;
    use crate::bucket::tests::{MyBucket, Ts};
    use crate::entity::tests::{make_device, DeviceType, Level, Nid, TDevice};
    use crate::scheduler::tests::{make_scheduler_with_2_devices, MyScheduler};
    use krabmaga::simulate;

    pub(crate) type MyNode = GNode<MyBucket, TDevice, Nid, DeviceType, Level, Ts>;

    pub(crate) fn as_node(device: TDevice) -> MyNode {
        let device_type = device.device_type.clone();
        GNode::new(device.id, device, device_type)
    }

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

    #[test]
    fn test_make_nodes() {
        let device_a = make_device(Nid::from(1), DeviceType::TypeA, 1);
        let node_a = as_node(device_a);
        assert_eq!(node_a.node_id, Nid::from(1));
        assert_eq!(node_a.kind, DeviceType::TypeA);
    }
}
