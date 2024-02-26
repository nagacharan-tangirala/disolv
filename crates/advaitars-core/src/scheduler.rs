use crate::agent::{Agent, AgentId, AgentImpl, AgentOrder};
use crate::bucket::{Bucket, TimeMS};
use crate::core::Core;
use hashbrown::HashMap;
use keyed_priority_queue::KeyedPriorityQueue;
use log::debug;
use typed_builder::TypedBuilder;

/// A trait used to represent a scheduler. A scheduler is used to schedule entities. The order
/// of calling the scheduler's functions is important to ensure the correct behavior of the engine.
/// Adding and removing entities should be handled in this trait.
pub trait Scheduler: Send + Sync {
    fn initialize(&mut self);
    fn activate(&mut self);
    fn collect_stats(&mut self);
    fn trigger(&mut self) -> TimeMS;
    fn terminate(&mut self);
}

#[derive(TypedBuilder)]
pub struct DefaultScheduler<A, B>
where
    A: Agent<B>,
    B: Bucket,
{
    pub core: Core<A, B>,
    pub agents: HashMap<AgentId, AgentImpl<A, B>>,
    pub duration: TimeMS,
    pub streaming_interval: TimeMS,
    pub step_size: TimeMS,
    pub output_interval: TimeMS,
    #[builder(default)]
    pub agent_queue: KeyedPriorityQueue<AgentId, AgentOrder>,
    #[builder(default = TimeMS::default())]
    pub now: TimeMS,
    #[builder(default = TimeMS::default())]
    pub streaming_step: TimeMS,
    #[builder(default = TimeMS::default())]
    pub output_step: TimeMS,
}

impl<A, B> DefaultScheduler<A, B>
where
    A: Agent<B>,
    B: Bucket,
{
    pub fn agent_of(&self, agent_id: &AgentId) -> &A {
        return &self
            .agents
            .get(agent_id)
            .expect("Agent not found in core")
            .agent;
    }

    #[inline]
    pub fn add_to_queue(&mut self, agent_id: AgentId, order: AgentOrder) {
        self.agent_queue.push(agent_id, order);
    }
}

impl<A, B> Scheduler for DefaultScheduler<A, B>
where
    A: Agent<B>,
    B: Bucket,
{
    fn initialize(&mut self) {
        for agent in self.agents.values_mut() {
            debug!("Adding agent {} to the core", agent.agent_id);
            self.core
                .add_agent(agent.agent_id, agent.agent.time_to_activation());
        }
    }

    fn activate(&mut self) {
        if self.core.agent_cache.contains_key(&self.now) {
            let agent_ids = self.core.agent_cache.remove(&self.now).unwrap();
            for agent_id in agent_ids.iter() {
                self.add_to_queue(*agent_id, self.agent_of(&agent_id).order());
                self.agents
                    .get_mut(agent_id)
                    .expect("Agent not found in core")
                    .agent
                    .activate();
            }
        }
    }

    fn collect_stats(&mut self) {
        for agent in self.agents.values() {
            if !agent.agent.is_deactivated() {
                self.core
                    .agent_stats
                    .insert(agent.agent_id, agent.agent.stats());
            }
        }
    }

    fn trigger(&mut self) -> TimeMS {
        self.core.bucket.before_agents(self.now);

        // This should be moved out of here.
        if self.now == self.streaming_step {
            self.core.bucket.stream_input(self.now);
            self.streaming_step += self.streaming_interval;
        }

        // This should be moved out of here.
        if self.now == self.output_step {
            self.core.bucket.stream_output(self.now);
            self.output_step += self.output_interval;
        }

        // Early return if the agent queue is empty.
        if self.agent_queue.is_empty() {
            self.core.bucket.after_agents();
            self.now += self.step_size;
            return self.now;
        }

        // Pop all the agents from the queue.
        let mut agent_ids: Vec<AgentId> = Vec::new();
        loop {
            if self.agent_queue.is_empty() {
                break;
            }
            match self.agent_queue.pop() {
                Some((agent_id, _)) => agent_ids.push(agent_id),
                None => panic!("Agent not found in core"),
            }
        }

        agent_ids.iter_mut().rev().for_each(|agent_id| {
            self.agents
                .get_mut(agent_id)
                .expect("Agent not found in core")
                .agent
                .stage_one(&mut self.core);
        });
        self.core.bucket.after_stage_one();

        agent_ids.iter_mut().for_each(|agent_id| {
            self.agents
                .get_mut(agent_id)
                .expect("Agent not found in core")
                .agent
                .stage_two_reverse(&mut self.core);
        });
        self.core.bucket.after_stage_two();

        agent_ids.iter_mut().rev().for_each(|agent_id| {
            self.agents
                .get_mut(agent_id)
                .expect("Agent not found in core")
                .agent
                .stage_three(&mut self.core);
        });
        self.core.bucket.after_stage_three();

        agent_ids.iter_mut().for_each(|agent_id| {
            self.agents
                .get_mut(agent_id)
                .expect("Agent not found in core")
                .agent
                .stage_four_reverse(&mut self.core);
        });
        self.core.bucket.after_stage_four();

        agent_ids.iter_mut().rev().for_each(|agent_id| {
            self.agents
                .get_mut(agent_id)
                .expect("Agent not found in core")
                .agent
                .stage_five(&mut self.core);
        });

        self.core.bucket.after_agents();

        // Reschedule the agents if not stopped.
        for agent_id in agent_ids.into_iter() {
            if self.agent_of(&agent_id).is_deactivated() {
                continue;
            }
            self.add_to_queue(agent_id, self.agent_of(&agent_id).order());
        }

        self.now += self.step_size;
        self.now
    }

    fn terminate(&mut self) {
        self.core.bucket.terminate(self.now);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::agent::tests::{make_device, DeviceType, TDevice};
    use crate::bucket::tests::MyBucket;
    use crate::core::tests::create_core;

    fn create_agent_map() -> HashMap<AgentId, AgentImpl<TDevice, MyBucket>> {
        let mut agents = HashMap::new();
        let device_a = make_device(AgentId::from(1), DeviceType::TypeA, 1);
        let device_b = make_device(AgentId::from(2), DeviceType::TypeB, 2);
        agents.insert(
            device_a.id(),
            AgentImpl::builder()
                .agent_id(device_a.id())
                .agent(device_a)
                .build(),
        );
        agents.insert(
            device_b.id(),
            AgentImpl::builder()
                .agent_id(device_b.id())
                .agent(device_b)
                .build(),
        );
        for i in 3..=100000 {
            let device = make_device(AgentId::from(i), DeviceType::TypeA, i as i32);
            agents.insert(
                device.id(),
                AgentImpl::builder()
                    .agent_id(device.id())
                    .agent(device)
                    .build(),
            );
        }
        agents
    }

    pub(crate) fn create_scheduler() -> DefaultScheduler<TDevice, MyBucket> {
        DefaultScheduler {
            core: create_core(),
            agents: create_agent_map(),
            agent_queue: KeyedPriorityQueue::new(),
            duration: TimeMS::from(1000),
            streaming_interval: TimeMS::from(10),
            streaming_step: TimeMS::from(0),
            output_interval: TimeMS::from(100),
            output_step: TimeMS::from(0),
            step_size: TimeMS::from(100),
            now: TimeMS::from(0),
        }
    }

    #[test]
    fn test_activate() {
        let mut scheduler = create_scheduler();
        scheduler.activate();
        assert_eq!(scheduler.agent_queue.len(), 2);
    }

    #[test]
    fn test_collect_stats() {
        let mut scheduler = create_scheduler();
        scheduler.collect_stats();
        assert_eq!(scheduler.core.agent_stats.len(), 2);
    }

    #[test]
    fn test_trigger() {
        let mut scheduler = create_scheduler();
        scheduler.activate();
        scheduler.trigger();
        assert_eq!(scheduler.now, TimeMS::from(100));
    }
}
