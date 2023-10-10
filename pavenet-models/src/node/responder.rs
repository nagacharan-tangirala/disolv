use crate::model::{NodeModel, TomlReadable};
use crate::node::payload::Payload;
use hashbrown::HashMap;
use pavenet_core::types::NodeId;
use rand::prelude::SliceRandom;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Responder {
    pub name: String,
}

impl TomlReadable for Responder {}

#[derive(Clone, Debug, Copy)]
pub enum ResponderType {
    Stats(StatsResponder),
}

#[derive(Clone, Debug, Copy)]
pub(crate) struct StatsResponder;

impl NodeModel for StatsResponder {
    type Input = Responder;
    fn to_input(&self) -> Responder {
        let name: String = "stats".to_string();
        Responder { name }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct DownlinkPayload {
    pub(crate) id: NodeId,
    pub(crate) latency_factor: u32,
}

impl StatsResponder {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn respond_to_vehicles(
        &self,
        veh_payloads: &Vec<Payload>,
        rsu_counts: usize,
    ) -> HashMap<NodeId, DownlinkPayload> {
        let mut veh_ids: Vec<NodeId> = veh_payloads
            .iter()
            .map(|p| p.sensor_data.node_info.id)
            .collect();
        veh_ids.shuffle(&mut rand::thread_rng());
        let crowd_latency = rsu_counts + veh_ids.len();

        let mut responses: HashMap<NodeId, DownlinkPayload> = HashMap::with_capacity(veh_ids.len());
        for (idx, veh_id) in veh_ids.iter().enumerate() {
            let response = DownlinkPayload {
                id: *veh_id,
                latency_factor: (crowd_latency + idx) as u32,
            };
            responses.entry(*veh_id).or_insert(response);
        }
        responses
    }
}
