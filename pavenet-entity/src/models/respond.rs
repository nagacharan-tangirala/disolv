use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct ResponderSettings {
    pub name: String,
}

#[derive(Clone, Debug, Copy)]
pub enum Responder {
    Stats(StatsResponder),
}

impl Responder {}

#[derive(Clone, Debug, Copy, Default)]
pub struct StatsResponder {}

impl StatsResponder {
    pub(crate) fn new(responder_settings: ResponderSettings) -> Self {
        Self {}
    }

    pub fn respond_to_vehicles(
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
                latency_factor: crowd_latency - idx,
            };
            responses.insert(*veh_id, response);
        }
        responses
    }
}
