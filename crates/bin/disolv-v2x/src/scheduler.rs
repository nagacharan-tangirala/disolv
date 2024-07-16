use std::cmp::Ordering;

use indexmap::IndexMap;
use keyed_priority_queue::KeyedPriorityQueue;
use log::debug;
use typed_builder::TypedBuilder;

use disolv_core::agent::{Agent, AgentId, AgentImpl, AgentOrder};
use disolv_core::bucket::{Bucket, TimeMS};
use disolv_core::core::Core;
use disolv_core::hashbrown::HashMap;

/// A trait used to represent a scheduler. A scheduler is used to schedule entities. The order
/// of calling the scheduler's functions is important to ensure the correct behavior of the engine.
/// Adding and removing entities should be handled in this trait.
pub trait Scheduler: Send {
    fn duration(&self) -> TimeMS;
    fn initialize(&mut self);
    fn activate(&mut self);
    fn collect_stats(&mut self);
    fn trigger(&mut self) -> TimeMS;
    fn terminate(self);
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
    fn duration(&self) -> TimeMS {
        self.duration
    }

    fn initialize(&mut self) {
        for agent in self.agents.values_mut() {
            debug!("Adding agent {} to the core", agent.agent_id);
            self.core
                .add_agent(agent.agent_id, agent.agent.time_to_activation());
        }
        self.core.bucket.initialize(self.now);
    }

    fn activate(&mut self) {
        if self.core.agent_cache.contains_key(&self.now) {
            let agent_ids = self.core.agent_cache.remove(&self.now).unwrap();
            for agent_id in agent_ids.iter() {
                self.add_to_queue(*agent_id, self.agent_of(agent_id).order());
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

    fn terminate(self) {
        self.core.bucket.terminate(self.now);
    }
}

#[derive(TypedBuilder)]
pub struct MapScheduler<A, B>
where
    A: Agent<B>,
    B: Bucket,
{
    pub core: Core<A, B>,
    pub active_agents: IndexMap<AgentId, AgentImpl<A, B>>,
    pub duration: TimeMS,
    pub streaming_interval: TimeMS,
    pub step_size: TimeMS,
    pub output_interval: TimeMS,
    pub inactive_agents: HashMap<AgentId, AgentImpl<A, B>>,
    pub deactivated: Vec<AgentId>,
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
        return &self
            .active_agents
            .get(agent_id)
            .expect("Agent not found in core")
            .agent;
    }

    fn agent_cmp(
        this_id: &AgentId,
        this_agent: &AgentImpl<A, B>,
        other_id: &AgentId,
        other_agent: &AgentImpl<A, B>,
    ) -> Ordering {
        if this_agent.agent.order() == other_agent.agent.order() {
            if this_id == other_id {
                panic!("This should never happen!");
            }
            if this_id > other_id {
                return Ordering::Greater;
            }
            return Ordering::Less;
        }
        if this_agent.agent.order() > other_agent.agent.order() {
            return Ordering::Greater;
        }
        return Ordering::Less;
    }
}

impl<A, B> Scheduler for MapScheduler<A, B>
where
    A: Agent<B>,
    B: Bucket,
{
    fn duration(&self) -> TimeMS {
        self.duration
    }

    fn initialize(&mut self) {
        for agent in self.inactive_agents.values_mut() {
            debug!("Adding agent {} to the core", agent.agent_id);
            self.core
                .add_agent(agent.agent_id, agent.agent.time_to_activation());
        }
        self.core.bucket.initialize(self.now);
    }

    fn activate(&mut self) {
        if self.core.agent_cache.contains_key(&self.now) {
            let agent_ids = self.core.agent_cache.remove(&self.now).unwrap();
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
                    .agent
                    .activate();
            }
            self.active_agents.sort_by(MapScheduler::agent_cmp);
        }
    }

    fn collect_stats(&mut self) {
        for agent in self.active_agents.values() {
            self.core
                .agent_stats
                .insert(agent.agent_id, agent.agent.stats());
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
        if self.active_agents.is_empty() {
            self.core.bucket.after_agents();
            self.now += self.step_size;
            return self.now;
        }

        self.active_agents
            .values_mut()
            .for_each(|agent_impl| agent_impl.agent.stage_one(&mut self.core));

        self.core.bucket.after_stage_one();

        self.active_agents
            .values_mut()
            .rev()
            .for_each(|agent_impl| agent_impl.agent.stage_two_reverse(&mut self.core));

        self.core.bucket.after_stage_two();

        self.active_agents
            .values_mut()
            .for_each(|agent_impl| agent_impl.agent.stage_three(&mut self.core));

        self.core.bucket.after_stage_three();

        self.active_agents
            .values_mut()
            .rev()
            .for_each(|agent_impl| agent_impl.agent.stage_four_reverse(&mut self.core));

        self.core.bucket.after_stage_four();

        self.active_agents
            .values_mut()
            .for_each(|agent_impl| agent_impl.agent.stage_five(&mut self.core));

        self.core.bucket.after_agents();

        self.deactivated = self
            .active_agents
            .values()
            .filter(|agent| agent.agent.is_deactivated())
            .map(|agent| agent.agent_id)
            .collect();

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
        self.core.bucket.terminate(self.now);
    }
}
