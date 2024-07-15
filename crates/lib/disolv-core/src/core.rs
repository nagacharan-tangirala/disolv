use crate::agent::{Agent, AgentId};
use crate::bucket::{Bucket, TimeMS};
use hashbrown::HashMap;

pub struct Core<A, B>
where
    A: Agent<B>,
    B: Bucket,
{
    pub bucket: B,
    pub agent_cache: HashMap<TimeMS, Vec<AgentId>>,
    pub agent_stats: HashMap<AgentId, A::AS>,
}

impl<A, B> Core<A, B>
where
    A: Agent<B>,
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

    pub fn stats_of(&self, agent_id: &AgentId) -> &A::AS {
        match self.agent_stats.get(agent_id) {
            Some(stats) => stats,
            None => panic!(
                "{}",
                format!("Agent stats missing for agent {}", agent_id).as_str()
            ),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::agent::tests::{make_device, DeviceType, TDevice};
    use crate::agent::AgentImpl;
    use crate::bucket::tests::MyBucket;

    pub(crate) fn create_core() -> Core<TDevice, MyBucket> {
        Core {
            bucket: MyBucket::default(),
            agent_cache: HashMap::new(),
            agent_stats: HashMap::new(),
        }
    }

    #[test]
    fn test_add_agent() {
        let mut core = create_core();
        let device_a = make_device(AgentId::from(1), DeviceType::TypeA, 1);
        let device_b = make_device(AgentId::from(2), DeviceType::TypeB, 2);
        core.add_agent(device_a.id(), TimeMS::from(0));
        core.add_agent(device_b.id(), TimeMS::from(0));
        assert_eq!(core.agent_cache.len(), 1);
        assert_eq!(core.agent_cache.get(&TimeMS::from(0)).unwrap().len(), 2);
        let device_c = make_device(AgentId::from(3), DeviceType::TypeA, 3);
        core.add_agent(device_c.id(), TimeMS::from(1));
        assert_eq!(core.agent_cache.len(), 2);
        assert_eq!(core.agent_cache.get(&TimeMS::from(1)).unwrap().len(), 1);
    }
}
