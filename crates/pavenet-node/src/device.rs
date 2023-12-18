use crate::bucket::DeviceBucket;
use log::debug;
use pavenet_core::entity::{NodeClass, NodeInfo, NodeOrder};
use pavenet_core::message::{DPayload, NodeContent, PayloadInfo};
use pavenet_core::message::{DResponse, DataSource, RxMetrics};
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

    fn talk_to_class(&mut self, target_class: &NodeClass, bucket: &mut DeviceBucket) {
        let link_options = match bucket.link_options_for(
            self.node_info.id,
            &self.node_info.node_type,
            target_class,
        ) {
            Some(links) => links,
            None => return,
        };
        let stats = bucket.stats_for(&link_options);
        let targets = self.models.select_links(link_options, target_class, &stats);

        let payload = self
            .models
            .composer
            .compose_payload(target_class, self.content);
        targets.into_iter().for_each(|target_link| {
            if let Some(target_node) = bucket.node_of(target_link.target) {
                let mut this_payload = payload.clone();

                debug!(
                    "Preparing to forward {} payloads to node {}",
                    &self.models.radio.rx_payloads.len(),
                    &target_node.content.node_info.id
                );
                let mut blobs =
                    prepare_blobs_to_fwd(&target_node.content, &self.models.radio.rx_payloads);
                debug!(
                    "In rx_payload size: {}, blobs: {}",
                    &self.models.radio.rx_payloads.len(),
                    blobs.len()
                );
                self.models
                    .composer
                    .append_blobs_to(&mut this_payload, &mut blobs);
                let prepared_payload = self
                    .models
                    .radio
                    .prepare_transfer(target_class, this_payload);
                self.transmit(prepared_payload, target_link, bucket);
            } else {
                debug!("Missing node ID {} ", target_link.target);
            }
        });
    }

    fn talk_to_peers(&mut self, bucket: &mut DeviceBucket) {
        let sl_links = match bucket.link_options_for(
            self.node_info.id,
            &self.node_info.node_type,
            &self.node_info.node_class,
        ) {
            Some(link_opts) => link_opts,
            None => return,
        };
        let stats = bucket.stats_for(&sl_links);
        let sl_targets = self
            .models
            .select_links(sl_links, &self.node_info.node_class, &stats);

        let payload = self
            .models
            .composer
            .compose_payload(&self.node_info.node_class, self.content);

        sl_targets.into_iter().for_each(|target_link| {
            if let Some(target_node) = bucket.node_of(target_link.target) {
                let mut this_payload = payload.clone();
                debug!(
                    "SL Preparing to forward {} payloads to node {}",
                    &self.models.sl_radio.sl_payloads.len(),
                    &target_node.content.node_info.id
                );
                let mut blobs =
                    prepare_blobs_to_fwd(&target_node.content, &self.models.sl_radio.sl_payloads);
                self.models
                    .composer
                    .append_blobs_to(&mut this_payload, &mut blobs);

                let prepared_payload = self.models.sl_radio.prepare_transfer(this_payload);
                self.transmit_sl(prepared_payload, target_link, bucket);
            } else {
                debug!("Missing node ID {} ", target_link.target);
            }
        });
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
        debug!("Stopping node: {}", self.node_info.id);
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
        let feasible_and_stats: (Vec<DPayload>, Vec<RxMetrics>) =
            self.models.rx_radio.complete_transfers(incoming);
        feasible_and_stats.1.iter().for_each(|rx_metrics| {
            bucket
                .resultant
                .add_rx_data(self.step, self.node_info.id, rx_metrics)
        });
        self.models
            .rx_radio
            .perform_actions(&self.compose_content(), feasible_and_stats.0)
    }

    fn find_targets(&self, target_class: &NodeClass, bucket: &mut DeviceBucket) -> Option<DLink> {
        let link_options = match bucket.link_options_for(
            self.node_info.id,
            &self.node_info.node_type,
            target_class,
        ) {
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
        debug!(
            "Uplink stage for node: {} id at step: {}",
            self.node_info.id, self.step
        );
        self.step = bucket.step;
        self.power_state = PowerState::On;
        self.set_mobility(bucket);

        let payloads_to_fwd = self.collect(bucket);

        for target_class in self.target_classes.clone().iter() {
            if let Some(target_link) = self.find_targets(target_class, bucket) {
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
            self.power_state = PowerState::Off;
            bucket.add_to_schedule(self.node_info.id);
            bucket.stop_node(self.node_info.id);
            bucket
                .devices
                .entry(self.node_info.id)
                .and_modify(|device| *device = self.clone())
                .or_insert(self.clone());
        }
        self.models.rx_radio.reset_rx();
    }
}
