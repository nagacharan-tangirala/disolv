use disolv_core::agent::AgentId;
use disolv_core::hashbrown::HashMap;
use disolv_core::message::{AgentState, GPayload, GResponse, Metadata, Reply, TxReport};

#[derive(Clone, Debug, Default)]
pub struct DataLake<A, M, R, T>
where
    A: AgentState,
    M: Metadata,
    R: Reply,
    T: TxReport,
{
    pub payloads: HashMap<AgentId, Vec<GPayload<A, M>>>,
    pub sl_payloads: HashMap<AgentId, Vec<GPayload<A, M>>>,
    pub responses: HashMap<AgentId, GResponse<R, T>>,
    pub sl_responses: HashMap<AgentId, GResponse<R, T>>,
}

impl<A, M, R, T> DataLake<A, M, R, T>
where
    A: AgentState,
    M: Metadata,
    R: Reply,
    T: TxReport,
{
    pub fn new() -> Self {
        Self {
            payloads: HashMap::new(),
            sl_payloads: HashMap::new(),
            responses: HashMap::new(),
            sl_responses: HashMap::new(),
        }
    }
    pub fn payloads_for(&mut self, agent_id: AgentId) -> Option<Vec<GPayload<A, M>>> {
        self.payloads.remove(&agent_id)
    }

    pub fn add_payload_to(&mut self, agent_id: AgentId, payload: GPayload<A, M>) {
        self.payloads.entry(agent_id).or_default().push(payload);
    }

    pub fn add_sl_payload_to(&mut self, agent_id: AgentId, payload: GPayload<A, M>) {
        self.sl_payloads.entry(agent_id).or_default().push(payload);
    }

    pub fn sl_payloads_for(&mut self, agent_id: AgentId) -> Option<Vec<GPayload<A, M>>> {
        self.sl_payloads.remove(&agent_id)
    }

    pub fn response_for(&mut self, agent_id: AgentId) -> Option<GResponse<R, T>> {
        self.responses.remove(&agent_id)
    }

    pub fn sl_response_for(&mut self, agent_id: AgentId) -> Option<GResponse<R, T>> {
        self.sl_responses.remove(&agent_id)
    }

    pub fn add_sl_response_to(&mut self, agent_id: AgentId, response: GResponse<R, T>) {
        self.sl_responses.entry(agent_id).or_insert(response);
    }

    pub fn add_response_to(&mut self, agent_id: AgentId, response: GResponse<R, T>) {
        self.responses.entry(agent_id).or_insert(response);
    }

    pub fn clean_payloads(&mut self) {
        self.payloads.clear();
        self.sl_payloads.clear();
    }

    pub fn clean_responses(&mut self) {
        self.responses.clear();
        self.sl_responses.clear();
    }
}
