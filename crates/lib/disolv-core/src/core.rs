use hashbrown::HashMap;

use crate::agent::{AgentId, AgentStats};
use crate::bucket::{Bucket, TimeMS};

pub struct Core<A, B>
where
    A: AgentStats,
    B: Bucket,
{
    pub bucket: B,
    pub agent_cache: HashMap<TimeMS, Vec<AgentId>>,
    pub agent_stats: HashMap<AgentId, A>,
}

impl<A, B> Core<A, B>
where
    A: AgentStats,
    B: Bucket,
{
    pub fn new(bucket: B) -> Core<A, B> {
        Core {
            bucket,
            agent_cache: HashMap::new(),
            agent_stats: HashMap::new(),
        }
    }

    pub fn add_agent(&mut self, agent_id: AgentId, time_to_add: TimeMS) {
        self.agent_cache
            .entry(time_to_add)
            .or_default()
            .push(agent_id);
    }

    pub fn stats_of(&self, agent_id: &AgentId) -> &A {
        match self.agent_stats.get(agent_id) {
            Some(stats) => stats,
            None => panic!(
                "{}",
                format!("Agent stats missing for agent {}", agent_id).as_str()
            ),
        }
    }
}
