use pavenet_core::message::DPayload;
use pavenet_core::message::DResponse;
use pavenet_engine::hashbrown::HashMap;
use pavenet_engine::node::NodeId;

pub type PayloadMap = HashMap<NodeId, Vec<DPayload>>; // TargetNodeId -> Payloads
pub type ResponseMap = HashMap<NodeId, DResponse>; // TargetNodeId -> Response

#[derive(Clone, Debug, Default)]
pub struct DataLake {
    pub payloads: PayloadMap,
    pub responses: ResponseMap,
}

impl DataLake {
    pub fn payloads_for(&mut self, node_id: NodeId) -> Option<Vec<DPayload>> {
        self.payloads.remove(&node_id)
    }

    pub fn add_payload_to(&mut self, node_id: NodeId, payload: DPayload) {
        self.payloads.entry(node_id).or_default().push(payload);
    }

    pub fn response_for(&mut self, node_id: NodeId) -> Option<DResponse> {
        self.responses.remove(&node_id)
    }

    pub fn add_response_to(&mut self, node_id: NodeId, response: DResponse) {
        self.responses.entry(node_id).or_insert(response);
    }

    pub fn clean_up_payloads(&mut self) {
        self.payloads.clear();
    }

    pub fn clean_up_responses(&mut self) {
        self.responses.clear();
    }
}
