use std::cmp::Ordering;

use crate::agent::{Agent, AgentId, AgentImpl};
use crate::bucket::{Bucket, TimeMS};
use crate::core::Core;
use crate::hashbrown::HashMap;
use crate::scheduler::Scheduler;
use indexmap::IndexMap;
use log::debug;
use typed_builder::TypedBuilder;

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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::agent::tests::{make_device, DeviceType, TDevice};
    use crate::bucket::tests::MyBucket;
    use crate::core::tests::create_core;

    pub(crate) fn create_map_scheduler() -> MapScheduler<TDevice, MyBucket> {
        let agents = IndexMap::with_capacity(1000000);
        let mut inactive_agents = HashMap::with_capacity(1000000);
        for i in 0..1000000 {
            let device = make_device(AgentId::from(i), DeviceType::TypeA, i as i32);
            inactive_agents.insert(
                device.id(),
                AgentImpl::builder()
                    .agent_id(device.id())
                    .agent(device)
                    .build(),
            );
        }
        MapScheduler {
            core: create_core(),
            active_agents: agents,
            inactive_agents,
            deactivated: Vec::with_capacity(100000),
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
    fn test_map_activate() {
        let mut scheduler = create_map_scheduler();
        scheduler.activate();
        assert_eq!(scheduler.active_agents.len(), 1000000);
    }

    #[test]
    fn test_map_collect_stats() {
        let mut scheduler = create_map_scheduler();
        scheduler.collect_stats();
        assert_eq!(scheduler.core.agent_stats.len(), 1000000);
    }

    #[test]
    fn test_map_trigger() {
        let mut scheduler = create_map_scheduler();
        scheduler.activate();
        scheduler.trigger();
        assert_eq!(scheduler.now, TimeMS::from(100));
    }
}
