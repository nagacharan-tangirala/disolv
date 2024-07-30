use disolv_core::agent::{AgentId, AgentProperties};
use disolv_core::hashbrown::HashMap;
use disolv_core::message::{ContentType, DataUnit, Metadata, Payload, QueryType};

#[derive(Clone, Debug, Default)]
pub struct DataLake<C, D, M, P, Q>
where
    C: ContentType,
    D: DataUnit<C>,
    M: Metadata,
    P: AgentProperties,
    Q: QueryType,
{
    pub payloads: HashMap<AgentId, Vec<Payload<C, D, M, P, Q>>>,
    pub sl_payloads: HashMap<AgentId, Vec<Payload<C, D, M, P, Q>>>,
}

impl<C, D, M, P, Q> DataLake<C, D, M, P, Q>
where
    C: ContentType,
    D: DataUnit<C>,
    M: Metadata,
    P: AgentProperties,
    Q: QueryType,
{
    pub fn new() -> Self {
        Self {
            payloads: HashMap::new(),
            sl_payloads: HashMap::new(),
        }
    }
    pub fn payloads_for(&mut self, agent_id: AgentId) -> Option<Vec<Payload<C, D, M, P, Q>>> {
        self.payloads.remove(&agent_id)
    }

    pub fn add_payload_to(&mut self, agent_id: AgentId, payload: Payload<C, D, M, P, Q>) {
        self.payloads.entry(agent_id).or_default().push(payload);
    }

    pub fn add_sl_payload_to(&mut self, agent_id: AgentId, payload: Payload<C, D, M, P, Q>) {
        self.sl_payloads.entry(agent_id).or_default().push(payload);
    }

    pub fn sl_payloads_for(&mut self, agent_id: AgentId) -> Option<Vec<Payload<C, D, M, P, Q>>> {
        self.sl_payloads.remove(&agent_id)
    }

    pub fn clean_payloads(&mut self) {
        self.payloads.clear();
        self.sl_payloads.clear();
    }
}
