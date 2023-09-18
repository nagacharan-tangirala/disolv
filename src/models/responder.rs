use crate::models::composer::UplinkPayload;
use crate::reader::activation::DeviceId;
use crate::utils::dyn_config::DataType;
use krabmaga::hashbrown::HashMap;
use rand::prelude::SliceRandom;

#[derive(Clone, Debug, Copy)]
pub(crate) enum ResponderType {
    Stats(StatsResponder),
}

#[derive(Clone, Debug, Copy)]
pub(crate) struct StatsResponder;

#[derive(Clone, Debug, Default)]
pub(crate) struct DownlinkPayload {
    pub(crate) id: DeviceId,
    pub(crate) latency_factor: u32,
    pub(crate) data_request: Vec<DataType>,
}

impl StatsResponder {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn respond_to_vehicles(
        &self,
        veh_payloads: &Vec<UplinkPayload>,
        rsu_counts: usize,
    ) -> HashMap<DeviceId, DownlinkPayload> {
        let mut veh_ids: Vec<DeviceId> = veh_payloads.iter().map(|p| p.id).collect();
        veh_ids.shuffle(&mut rand::thread_rng());
        let crowd_latency = rsu_counts + veh_ids.len();

        let mut responses: HashMap<DeviceId, DownlinkPayload> =
            HashMap::with_capacity(veh_ids.len());
        for (idx, veh_id) in veh_ids.iter().enumerate() {
            let response = DownlinkPayload {
                id: *veh_id,
                latency_factor: (crowd_latency + idx) as u32,
                data_request: Vec::new(),
            };
            responses.entry(*veh_id).or_insert(response);
        }
        responses
    }
}
