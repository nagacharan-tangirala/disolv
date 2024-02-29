use crate::net::message::DPayload;
use crate::net::message::DResponse;
use disolv_core::agent::AgentId;
use disolv_core::hashbrown::HashMap;

pub type PayloadMap = HashMap<AgentId, Vec<DPayload>>;
// TargetNodeId -> Payloads
pub type ResponseMap = HashMap<AgentId, DResponse>; // TargetNodeId -> Response

#[derive(Clone, Debug, Default)]
pub struct DataLake {
    pub payloads: PayloadMap,
    pub sl_payloads: PayloadMap,
    pub responses: ResponseMap,
    pub sl_responses: ResponseMap,
}

impl DataLake {
    pub fn payloads_for(&mut self, node_id: AgentId) -> Option<Vec<DPayload>> {
        self.payloads.remove(&node_id)
    }

    pub fn add_payload_to(&mut self, node_id: AgentId, payload: DPayload) {
        self.payloads.entry(node_id).or_default().push(payload);
    }

    pub fn add_sl_payload_to(&mut self, node_id: AgentId, payload: DPayload) {
        self.sl_payloads.entry(node_id).or_default().push(payload);
    }

    pub fn sl_payloads_for(&mut self, node_id: AgentId) -> Option<Vec<DPayload>> {
        self.sl_payloads.remove(&node_id)
    }

    pub fn response_for(&mut self, node_id: AgentId) -> Option<DResponse> {
        self.responses.remove(&node_id)
    }

    pub fn sl_response_for(&mut self, node_id: AgentId) -> Option<DResponse> {
        self.sl_responses.remove(&node_id)
    }

    pub fn add_sl_response_to(&mut self, node_id: AgentId, response: DResponse) {
        self.sl_responses.entry(node_id).or_insert(response);
    }

    pub fn add_response_to(&mut self, node_id: AgentId, response: DResponse) {
        self.responses.entry(node_id).or_insert(response);
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
