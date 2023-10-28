use hashbrown::HashMap;
use pavenet_core::node_info::id::NodeId;
use pavenet_core::payload::DPayload;
use pavenet_core::response::TResponse;
use typed_builder::TypedBuilder;

pub type PayloadMap = HashMap<NodeId, Vec<DPayload>>; // TargetNodeId -> Payloads
pub type ResponseMap = HashMap<NodeId, TResponse>; // TargetNodeId -> Response

#[derive(Clone, Debug, Default, TypedBuilder)]
pub struct DataLake {
    pub payloads: PayloadMap,
    pub responses: ResponseMap,
}

impl DataLake {
    pub fn payloads_for(&mut self, node_id: NodeId) -> Vec<DPayload> {
        self.payloads.remove(&node_id).unwrap_or_default()
    }

    pub fn add_payload_to(&mut self, node_id: NodeId, payload: DPayload) {
        self.payloads
            .entry(node_id)
            .or_insert_with(Vec::new)
            .push(payload);
    }

    pub fn responses_for(&mut self, node_id: NodeId) -> TResponse {
        match self.responses.remove(&node_id) {
            Some(response) => response,
            None => panic!("No response for node_id: {:?}", node_id),
        }
    }

    pub fn add_response_to(&mut self, node_id: NodeId, response: TResponse) {
        self.responses.entry(node_id).or_insert(response);
    }
}
