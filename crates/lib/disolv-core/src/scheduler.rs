use std::cmp::Ordering;

use indexmap::IndexMap;
use keyed_priority_queue::KeyedPriorityQueue;
use log::debug;
use typed_builder::TypedBuilder;

use crate::agent::{Agent, AgentId, AgentOrder};
use crate::bucket::{Bucket, TimeMS};
use crate::hashbrown::HashMap;

/// A trait used to represent a scheduler. A scheduler is used to schedule entities. The order
/// of calling the scheduler's functions is important to ensure the correct behavior of the engine.
/// Adding and removing entities should be handled in this trait.
pub trait Scheduler<B: Bucket>: Send {
    fn duration(&self) -> TimeMS;
    fn initialize(&mut self);
    fn activate(&mut self);
    fn trigger(&mut self) -> TimeMS;
    fn terminate(self);
}

#[derive(TypedBuilder)]
pub struct DefaultScheduler<A, B>
where
    A: Agent<B>,
    B: Bucket,
{
    pub bucket: B,
    pub agents: HashMap<AgentId, A>,
    pub duration: TimeMS,
    pub streaming_interval: TimeMS,
    pub step_size: TimeMS,
    pub output_interval: TimeMS,
    #[builder(default)]
    pub agent_cache: HashMap<TimeMS, Vec<AgentId>>,
    #[builder(default)]
    pub agent_queue: KeyedPriorityQueue<AgentId, AgentOrder>,
    #[builder(default = TimeMS::default())]
    pub now: TimeMS,
    #[builder(default = TimeMS::default())]
    pub streaming_step: TimeMS,
    #[builder(default = TimeMS::default())]
    pub output_step: TimeMS,
    #[builder(default)]
    pub _marker: std::marker::PhantomData<fn() -> B>,
}

impl<A, B> DefaultScheduler<A, B>
where
    A: Agent<B>,
    B: Bucket,
{
    pub fn agent_of(&self, agent_id: &AgentId) -> &A {
        return &self.agents.get(agent_id).expect("Agent not found in core");
    }

    #[inline]
    pub fn add_to_queue(&mut self, agent_id: AgentId, order: AgentOrder) {
        self.agent_queue.push(agent_id, order);
    }
}

impl<A, B> Scheduler<B> for DefaultScheduler<A, B>
where
    A: Agent<B>,
    B: Bucket,
{
    fn duration(&self) -> TimeMS {
        self.duration
    }

    fn initialize(&mut self) {
        for agent in self.agents.values_mut() {
            debug!("Adding agent {} to the scheduler", agent.id());
            self.agent_cache
                .entry(agent.time_of_activation())
                .or_default()
                .push(agent.id());
        }
        self.bucket.initialize(self.now);
    }

    fn activate(&mut self) {
        if self.agent_cache.contains_key(&self.now) {
            let agent_ids = self.agent_cache.remove(&self.now).unwrap();
            for agent_id in agent_ids.iter() {
                self.add_to_queue(*agent_id, self.agent_of(agent_id).order());
                self.agents
                    .get_mut(agent_id)
                    .expect("Agent not found in core")
                    .activate(&mut self.bucket);
            }
        }
    }

    fn trigger(&mut self) -> TimeMS {
        self.bucket.before_agents(self.now);

        // This should be moved out of here.
        if self.now == self.streaming_step {
            self.bucket.stream_input();
            self.streaming_step += self.streaming_interval;
        }

        // This should be moved out of here.
        if self.now == self.output_step {
            self.bucket.stream_output();
            self.output_step += self.output_interval;
        }

        // Early return if the agent queue is empty.
        if self.agent_queue.is_empty() {
            self.bucket.after_agents();
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
                .stage_one(&mut self.bucket);
        });
        self.bucket.after_stage_one();

        agent_ids.iter_mut().for_each(|agent_id| {
            self.agents
                .get_mut(agent_id)
                .expect("Agent not found in core")
                .stage_two_reverse(&mut self.bucket);
        });
        self.bucket.after_stage_two();

        agent_ids.iter_mut().rev().for_each(|agent_id| {
            self.agents
                .get_mut(agent_id)
                .expect("Agent not found in core")
                .stage_three(&mut self.bucket);
        });
        self.bucket.after_stage_three();

        agent_ids.iter_mut().for_each(|agent_id| {
            self.agents
                .get_mut(agent_id)
                .expect("Agent not found in core")
                .stage_four_reverse(&mut self.bucket);
        });
        self.bucket.after_stage_four();

        agent_ids.iter_mut().rev().for_each(|agent_id| {
            self.agents
                .get_mut(agent_id)
                .expect("Agent not found in core")
                .stage_five(&mut self.bucket);
        });

        self.bucket.after_agents();

        for agent_id in agent_ids.into_iter() {
            // Reschedule the agent if not stopped.
            if !self.agent_of(&agent_id).is_deactivated() {
                self.add_to_queue(agent_id, self.agent_of(&agent_id).order());
            }

            // If agent needs a later activation, add it to cache.
            let agent = self
                .agents
                .get_mut(&agent_id)
                .expect("Agent not found in core");
            if agent.has_activation() {
                self.agent_cache
                    .entry(agent.time_of_activation())
                    .or_default()
                    .push(agent.id());
            }
        }

        self.now += self.step_size;
        self.now
    }

    fn terminate(self) {
        self.bucket.terminate();
    }
}

#[derive(TypedBuilder)]
pub struct MapScheduler<A, B>
where
    A: Agent<B>,
    B: Bucket,
{
    pub bucket: B,
    pub active_agents: IndexMap<AgentId, A>,
    pub duration: TimeMS,
    pub streaming_interval: TimeMS,
    pub step_size: TimeMS,
    pub output_interval: TimeMS,
    pub inactive_agents: HashMap<AgentId, A>,
    pub deactivated: Vec<AgentId>,
    #[builder(default)]
    pub agent_cache: HashMap<TimeMS, Vec<AgentId>>,
    #[builder(default = TimeMS::default())]
    pub now: TimeMS,
    #[builder(default = TimeMS::default())]
    pub streaming_step: TimeMS,
    #[builder(default = TimeMS::default())]
    pub output_step: TimeMS,
}

impl<A, B> MapScheduler<A, B>
where
    A: Agent<B>,
    B: Bucket,
{
    pub fn agent_of(&self, agent_id: &AgentId) -> &A {
        return self
            .active_agents
            .get(agent_id)
            .expect("Agent not found in core");
    }

    fn agent_cmp(
        this_id: &AgentId,
        this_agent: &A,
        other_id: &AgentId,
        other_agent: &A,
    ) -> Ordering {
        if this_agent.order() == other_agent.order() {
            if this_id == other_id {
                panic!("This should never happen!");
            }
            if this_id > other_id {
                return Ordering::Greater;
            }
            return Ordering::Less;
        }
        if this_agent.order() > other_agent.order() {
            return Ordering::Greater;
        }
        return Ordering::Less;
    }
}

impl<A, B> Scheduler<B> for MapScheduler<A, B>
where
    A: Agent<B>,
    B: Bucket,
{
    fn duration(&self) -> TimeMS {
        self.duration
    }

    fn initialize(&mut self) {
        for agent in self.inactive_agents.values_mut() {
            debug!("Adding agent {} to the core", agent.id());
            self.agent_cache
                .entry(agent.time_of_activation())
                .or_default()
                .push(agent.id());
        }
        self.bucket.initialize(self.now);
    }

    fn activate(&mut self) {
        if self.agent_cache.contains_key(&self.now) {
            let agent_ids = self.agent_cache.remove(&self.now).unwrap();
            for agent_id in agent_ids.into_iter() {
                self.active_agents.insert(
                    agent_id,
                    self.inactive_agents
                        .remove(&agent_id)
                        .expect("missing agent"),
                );
                self.active_agents
                    .get_mut(&agent_id)
                    .expect("agent not found")
                    .activate(&mut self.bucket);
            }
            self.active_agents.sort_by(MapScheduler::agent_cmp);
        }
    }

    fn trigger(&mut self) -> TimeMS {
        self.bucket.before_agents(self.now);

        // This should be moved out of here.
        if self.now == self.streaming_step {
            self.bucket.stream_input();
            self.streaming_step += self.streaming_interval;
        }

        // This should be moved out of here.
        if self.now == self.output_step {
            self.bucket.stream_output();
            self.output_step += self.output_interval;
        }

        // Early return if the agent queue is empty.
        if self.active_agents.is_empty() {
            self.bucket.after_agents();
            self.now += self.step_size;
            return self.now;
        }

        self.active_agents
            .values_mut()
            .for_each(|agent| agent.stage_one(&mut self.bucket));

        self.bucket.after_stage_one();

        self.active_agents
            .values_mut()
            .rev()
            .for_each(|agent| agent.stage_two_reverse(&mut self.bucket));

        self.bucket.after_stage_two();

        self.active_agents
            .values_mut()
            .for_each(|agent| agent.stage_three(&mut self.bucket));

        self.bucket.after_stage_three();

        self.active_agents
            .values_mut()
            .rev()
            .for_each(|agent| agent.stage_four_reverse(&mut self.bucket));

        self.bucket.after_stage_four();

        self.active_agents
            .values_mut()
            .for_each(|agent| agent.stage_five(&mut self.bucket));

        self.bucket.after_agents();

        // Get agent IDs that are deactivated at this time step.
        self.deactivated = self
            .active_agents
            .values()
            .filter(|agent| agent.is_deactivated())
            .map(|agent| agent.id())
            .collect();

        // Add the agent next activation time to agent cache if available.
        self.deactivated.iter().for_each(|agent_id| {
            let agent = self
                .active_agents
                .get_mut(agent_id)
                .expect("agent is missing");
            if agent.has_activation() {
                self.agent_cache
                    .entry(agent.time_of_activation())
                    .or_default()
                    .push(agent.id());
            }
        });

        // Move the deactivated agents from active agent map to inactive agent map.
        self.deactivated.iter().for_each(|inactive| {
            self.inactive_agents.insert(
                *inactive,
                self.active_agents
                    .swap_remove(inactive)
                    .expect("missing agent"),
            );
        });
        self.deactivated.clear();

        self.now += self.step_size;
        self.now
    }

    fn terminate(self) {
        self.bucket.terminate();
    }
}
