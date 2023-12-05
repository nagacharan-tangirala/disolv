use crate::bucket::DeviceBucket;
use log::debug;
use pavenet_core::entity::{NodeClass, NodeInfo, NodeOrder};
use pavenet_core::message::{DPayload, NodeContent, PayloadInfo};
use pavenet_core::message::{DResponse, DataSource, TransferMetrics};
use pavenet_core::mobility::MapState;
use pavenet_core::power::{PowerManager, PowerState};
use pavenet_core::radio::{DLink, LinkProperties};
use pavenet_engine::bucket::TimeMS;
use pavenet_engine::entity::{Entity, Movable, Schedulable, Tiered};
use pavenet_engine::message::Responder;
use pavenet_engine::message::Transmitter;
use pavenet_engine::node::GNode;
use pavenet_engine::radio::{RxChannel, TxChannel};
use pavenet_models::compose::Composer;
use pavenet_models::radio::{RxRadio, TxRadio};
use pavenet_models::reply::Replier;
use pavenet_models::select::Selector;
use typed_builder::TypedBuilder;

pub type TNode = GNode<DeviceBucket, Device, NodeOrder>;

#[derive(Debug, Clone, TypedBuilder)]
pub struct DeviceModel {
    pub power: PowerManager,
    pub rx_radio: RxRadio,
    pub tx_radio: TxRadio,
    pub composer: Composer,
    pub replier: Replier,
    pub selector: Selector,
}

#[derive(Clone, Debug, TypedBuilder)]
pub struct Device {
    pub node_info: NodeInfo,
    pub models: DeviceModel,
    #[builder(default)]
    pub target_classes: Vec<NodeClass>,
    #[builder(default)]
    pub step: TimeMS,
    #[builder(default)]
    pub power_state: PowerState,
    #[builder(default)]
    pub map_state: MapState,
}

impl Device {
    fn compose_content(&self) -> NodeContent {
        NodeContent {
            node_info: self.node_info,
            map_state: self.map_state,
        }
    }

    fn compose_payload(&self, target_class: &NodeClass) -> DPayload {
        let node_content = self.compose_content();
        self.models
            .composer
            .compose_payload(target_class, node_content)
    }
}

impl Tiered for Device {
    type T = NodeOrder;

    fn tier(&self) -> NodeOrder {
        self.node_info.node_order
    }

    fn set_tier(&mut self, order: NodeOrder) {
        self.node_info.node_order = order;
    }
}

impl Movable<DeviceBucket> for Device {
    type M = MapState;

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

impl Schedulable for Device {
    fn stop(&mut self) {
        self.power_state = PowerState::Off;
    }

    fn is_stopped(&self) -> bool {
        self.power_state == PowerState::Off
    }

    fn time_to_add(&mut self) -> TimeMS {
        self.models.power.pop_time_to_on()
    }
}

impl Transmitter<DeviceBucket, LinkProperties, PayloadInfo, NodeContent> for Device {
    type NodeClass = NodeClass;

    fn collect(&mut self, bucket: &mut DeviceBucket) -> Vec<DPayload> {
        let incoming = match bucket.data_lake.payloads_for(self.node_info.id) {
            Some(incoming) => incoming,
            None => return Vec::new(),
        };
        let feasible = self.models.rx_radio.complete_transfers(incoming);
        self.models
            .rx_radio
            .perform_actions(&self.compose_content(), feasible)
    }

    fn find_target(&self, target_class: &NodeClass, bucket: &mut DeviceBucket) -> Option<DLink> {
        debug!("Finding target for {} at {}", self.node_info.id, self.step);
        let link_options = match bucket.link_options_for(self.node_info.id, target_class) {
            Some(links) => links,
            None => return None,
        };
        let stats = bucket.stats_for(&link_options);
        let target_link = self.models.selector.select_target(link_options, &stats);
        Some(target_link)
    }

    fn transmit(&mut self, payload: DPayload, link: DLink, bucket: &mut DeviceBucket) {
        debug!(
            "Transmitting from {} to {} at {}",
            self.node_info.id, link.target, self.step
        );
        bucket.resultant.add_tx_data(self.step, &payload);
        bucket.data_lake.add_payload_to(link.target, payload)
    }
}

impl Responder<DeviceBucket, DataSource, TransferMetrics> for Device {
    fn receive(&mut self, bucket: &mut DeviceBucket) -> Option<DResponse> {
        let response = match bucket.data_lake.response_for(self.node_info.id) {
            Some(response) => response,
            None => return None,
        };
        Some(response)
    }

    fn respond(&mut self, response: Option<DResponse>, bucket: &mut DeviceBucket) {
        for (node_id, transfer_stats) in self.models.rx_radio.transfer_stats().into_iter() {
            let this_response = self
                .models
                .replier
                .compose_response(response.clone(), transfer_stats);
            bucket.data_lake.add_response_to(node_id, this_response);
        }
    }
}

impl Entity<DeviceBucket, NodeOrder> for Device {
    fn uplink_stage(&mut self, bucket: &mut DeviceBucket) {
        debug!("Uplink stage for node: {} id", self.node_info.id);
        self.step = bucket.step;
        self.power_state = PowerState::On;
        self.set_mobility(bucket);

        let payloads_to_fwd = self.collect(bucket);

        for target_class in self.target_classes.clone().iter() {
            if let Some(target_link) = self.find_target(target_class, bucket) {
                let mut payload = self.compose_payload(target_class);
                let target_node_data = match bucket.node_of(target_link.target) {
                    Some(node) => node.compose_content(),
                    None => {
                        debug!("Node {} not found", target_link.target);
                        continue;
                    }
                };
                let blobs = self
                    .models
                    .tx_radio
                    .prepare_blobs_to_fwd(&target_node_data, &payloads_to_fwd);

                payload.metadata.data_blobs.extend(blobs);
                let prepared_payload = self.models.tx_radio.prepare_transfer(target_class, payload);
                self.transmit(prepared_payload, target_link, bucket);
            } else {
                debug!(
                    "No target link found for id: {} at step: {}",
                    self.node_info.id, self.step
                );
            }
        }

        bucket
            .devices
            .entry(self.node_info.id)
            .and_modify(|device| *device = self.clone())
            .or_insert(self.clone());
    }

    fn downlink_stage(&mut self, bucket: &mut DeviceBucket) {
        let response = self.receive(bucket);
        self.respond(response, bucket);
        if self.step == self.models.power.peek_time_to_off() {
            debug!("Removing node: {} at {}", self.node_info.id, self.step);
            bucket.add_to_schedule(self.node_info.id);
            bucket.stop_node(self.node_info.id);
        }
        self.models.rx_radio.reset_rx();
    }
}
