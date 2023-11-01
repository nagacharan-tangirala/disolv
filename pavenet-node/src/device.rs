use crate::bucket::DeviceBucket;
use crate::d_model::DeviceModel;
use crate::models::power::PowerState;
use pavenet_core::bucket::TimeS;
use pavenet_core::entity::class::NodeClass;
use pavenet_core::entity::id::NodeId;
use pavenet_core::entity::kind::NodeType;
use pavenet_core::entity::NodeInfo;
use pavenet_core::mobility::MapState;
use pavenet_core::payload::{DPayload, DataType, NodeContent, PayloadInfo};
use pavenet_core::response::{DResponse, DataSource, TransferMetrics};
use pavenet_engine::engine::GNode;
use pavenet_engine::entity::{Entity, Movable, Schedulable, Tiered};
use pavenet_engine::payload::Transmitter;
use pavenet_engine::radio::{Channel, OutgoingStats};
use pavenet_engine::response::Responder;
use typed_builder::TypedBuilder;

pub type TNode = GNode<DeviceBucket, Device, NodeId, NodeType, NodeClass, TimeS>;

#[derive(Clone, Debug, TypedBuilder)]
pub struct Device {
    pub node_info: NodeInfo,
    pub models: DeviceModel,
    pub map_state: MapState,
    pub power_state: PowerState,
    pub target_classes: Vec<NodeClass>,
    pub step: TimeS,
}

impl Device {
    fn compose_content(&self) -> NodeContent {
        NodeContent {
            node_info: self.node_info,
            map_state: self.map_state,
        }
    }
}

impl Tiered<NodeClass> for Device {
    fn tier(&self) -> NodeClass {
        self.node_info.node_class
    }

    fn set_tier(&mut self, tier: NodeClass) {
        self.node_info.node_class = tier;
    }
}

impl Movable<DeviceBucket, MapState, TimeS> for Device {
    fn mobility(&self) -> &MapState {
        &self.map_state
    }

    fn set_mobility(&mut self, bucket: &mut DeviceBucket) {
        self.map_state = match bucket.positions_for(self.node_info.id, &self.node_info.node_type) {
            Some(mobility) => mobility,
            None => self.map_state,
        };
    }
}

impl Schedulable<TimeS> for Device {
    fn stop(&mut self) {
        self.power_state = PowerState::Off;
    }

    fn is_stopped(&self) -> bool {
        self.power_state == PowerState::Off
    }

    fn time_to_add(&mut self) -> TimeS {
        self.models.power.pop_time_to_on()
    }
}

impl Transmitter<DeviceBucket, NodeContent, NodeType, PayloadInfo, DataType, NodeClass, TimeS>
    for Device
{
    fn collect(&mut self, bucket: &mut DeviceBucket) -> Vec<DPayload> {
        bucket.data_lake.payloads_for(self.node_info.id)
    }

    fn payloads_to_forward(
        &mut self,
        bucket: &mut DeviceBucket,
        payloads: Vec<DPayload>,
    ) -> Vec<DPayload> {
        let feasible = self.models.radio.can_transfer(payloads);
        let to_forward = self.models.radio.apply_tx_rules(&bucket.rules, feasible);
        return to_forward;
    }

    fn compose(&mut self, target_class: &NodeClass, gathered: &Vec<DPayload>) -> DPayload {
        return self.models.composer.compose_payload(
            target_class,
            self.compose_content(),
            gathered,
        );
    }

    fn transmit(&mut self, target_type: &NodeType, payload: DPayload, bucket: &mut DeviceBucket) {
        let link_options = match bucket.link_options_for(self.node_info.id, target_type) {
            Some(links) => links,
            None => return,
        };
        let stats = bucket.stats_for(&link_options.link_opts);
        let target_link = self.models.selector.select_target(link_options, &stats);
        self.models.radio.out_stats.update(&payload.metadata);
        bucket.data_lake.add_payload_to(target_link.target, payload)
    }
}

impl Responder<DeviceBucket, DataSource, TransferMetrics, DataType, NodeClass, TimeS> for Device {
    fn receive(&mut self, bucket: &mut DeviceBucket) -> DResponse {
        bucket.data_lake.response_for(self.node_info.id)
    }

    fn process(&mut self, response: DResponse) -> DResponse {
        match response.content {
            Some(ref queries) => self.models.composer.update_sources(queries),
            None => (),
        }
        response
    }

    fn respond(&mut self, response: DResponse, bucket: &mut DeviceBucket) {
        for (node_id, transfer_stats) in self.models.radio.transfer_stats().into_iter() {
            let response = self
                .models
                .responder
                .compose_response(response.clone(), transfer_stats);
            bucket.data_lake.add_response_to(node_id, response);
        }
    }
}

impl Entity<DeviceBucket, NodeClass, TimeS> for Device {
    fn uplink_stage(&mut self, bucket: &mut DeviceBucket) {
        self.step = bucket.step;
        self.set_mobility(bucket);

        let incoming = self.collect(bucket);
        let to_forward = self.payloads_to_forward(bucket, incoming);

        let target_classes: Vec<NodeClass> = self.target_classes.to_owned();
        let target_types: Vec<NodeType> =
            target_classes.iter().map(|x| bucket.kind_for(x)).collect();

        for (target_type, target_class) in target_types.iter().zip(target_classes.iter()) {
            let payload = self.compose(target_class, &to_forward);
            self.transmit(target_type, payload, bucket);
        }

        if self.step == self.models.power.peek_time_to_off() {
            bucket.add_to_schedule(self.node_info.id);
            bucket.stop_node(self.node_info.id);
        }

        bucket
            .devices
            .entry(self.node_info.id)
            .or_insert(self.clone());
    }

    fn downlink_stage(&mut self, bucket: &mut DeviceBucket) {
        let response = self.receive(bucket);
        let processed_response = self.process(response);
        self.respond(processed_response, bucket);

        if self.step == self.models.power.peek_time_to_off() {
            bucket.add_to_schedule(self.node_info.id);
            bucket.stop_node(self.node_info.id);
        }
    }
}
