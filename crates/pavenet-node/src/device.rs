use crate::bucket::DeviceBucket;
use log::debug;
use pavenet_core::entity::{NodeClass, NodeInfo, NodeOrder};
use pavenet_core::message::{DPayload, NodeContent, PayloadInfo};
use pavenet_core::message::{DResponse, DataSource, RxMetrics};
use pavenet_core::mobility::MapState;
use pavenet_core::power::{PowerManager, PowerState};
use pavenet_core::radio::{DLink, InDataStats, LinkProperties};
use pavenet_engine::bucket::TimeMS;
use pavenet_engine::entity::{Entity, Movable, Schedulable, Tiered};
use pavenet_engine::message::Transmitter;
use pavenet_engine::message::{Receiver, Responder};
use pavenet_engine::node::GNode;
use pavenet_engine::radio::{Channel, SlChannel};
use pavenet_models::actions::prepare_blobs_to_fwd;
use pavenet_models::compose::Composer;
use pavenet_models::radio::{Radio, SlRadio};
use pavenet_models::reply::Replier;
use pavenet_models::select::Selector;
use typed_builder::TypedBuilder;

pub type TNode = GNode<DeviceBucket, Device, NodeOrder>;

#[derive(Debug, Clone, TypedBuilder)]
pub struct DeviceModel {
    pub power: PowerManager,
    pub radio: Radio,
    pub sl_radio: SlRadio,
    pub composer: Composer,
    pub replier: Replier,
    pub selector: Vec<(NodeClass, Selector)>,
}

impl DeviceModel {
    fn select_links(
        &self,
        link_options: Vec<DLink>,
        target_class: &NodeClass,
        stats: &Vec<Option<&InDataStats>>,
    ) -> Vec<DLink> {
        for selectors in self.selector.iter() {
            if selectors.0 == *target_class {
                return selectors.1.do_selection(link_options, stats);
            }
        }
        return vec![];
    }
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
    #[builder(default)]
    pub content: NodeContent,
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
        self.map_state = bucket
            .positions_for(self.node_info.id, &self.node_info.node_type)
            .unwrap_or(self.map_state);
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

    fn transmit(&self, payload: DPayload, target_link: DLink, bucket: &mut DeviceBucket) {
        bucket
            .resultant
            .add_tx_data(self.step, &target_link, &payload);
        debug!(
            "Transmitting payload from node {} to node {} with blobs {}",
            payload.node_state.node_info.id.as_u32(),
            target_link.target.as_u32(),
            payload.metadata.data_blobs.len()
        );
        bucket.data_lake.add_payload_to(target_link.target, payload);
    }

    fn transmit_sl(&self, payload: DPayload, target_link: DLink, bucket: &mut DeviceBucket) {
        bucket
            .resultant
            .add_tx_data(self.step, &target_link, &payload);
        debug!(
            "Transmitting payload from node {} to node {}",
            payload.node_state.node_info.id.as_u32(),
            target_link.target.as_u32()
        );
        bucket
            .data_lake
            .add_sl_payload_to(target_link.target, payload);
    }
}

impl Receiver<DeviceBucket, PayloadInfo, NodeContent> for Device {
    type C = NodeClass;

    fn receive(&mut self, bucket: &mut DeviceBucket) {
        let payloads = match bucket.data_lake.payloads_for(self.node_info.id) {
            Some(payloads) => payloads,
            None => return,
        };
        debug!(
            "Receiving payload at node {} at step {} payloads: {}",
            self.node_info.id,
            self.step,
            payloads.len()
        );
        self.models.radio.do_receive(&self.content, payloads);

        self.models.radio.rx_metrics.iter().for_each(|rx_metrics| {
            bucket
                .resultant
                .add_rx_data(self.step, self.node_info.id, rx_metrics)
        });
    }

    fn receive_sl(&mut self, bucket: &mut DeviceBucket) {
        let sl_payloads = match bucket.data_lake.sl_payloads_for(self.node_info.id) {
            Some(payloads) => payloads,
            None => return,
        };

        debug!(
            "Receiving SL payload at node {} at step {} payloads: {}",
            self.node_info.id,
            self.step,
            sl_payloads.len()
        );
        self.models.sl_radio.do_receive(&self.content, sl_payloads);

        self.models
            .sl_radio
            .sl_metrics
            .iter()
            .for_each(|sl_metrics| {
                bucket
                    .resultant
                    .add_sl_data(self.step, self.node_info.id, sl_metrics)
            });
    }
}

impl Responder<DeviceBucket, DataSource, RxMetrics> for Device {
    fn respond(&mut self, response: Option<DResponse>, bucket: &mut DeviceBucket) {
        for transfer_stats in self.models.radio.transfer_stats().into_iter() {
            let this_response = self
                .models
                .replier
                .compose_response(response.clone(), transfer_stats);
            bucket
                .data_lake
                .add_response_to(transfer_stats.from_node, this_response);
        }
    }

    fn respond_sl(&mut self, response: Option<DResponse>, bucket: &mut DeviceBucket) {
        // send to talk_to_peers
    }
}

impl Entity<DeviceBucket, NodeOrder> for Device {
    fn uplink_stage(&mut self, bucket: &mut DeviceBucket) {
        self.power_state = PowerState::On;
        self.step = bucket.step;
        self.set_mobility(bucket);
        self.content = self.compose_content();

        debug!(
            "Uplink stage for node: {} id at step: {}",
            self.node_info.id, self.step
        );
        self.models.flow.reset();

        // Receive data from the downstream nodes.
        let mut rx_payloads = self.receive(bucket).unwrap_or_else(Vec::new);
        self.models.flow.register_incoming(&rx_payloads);

        // Apply actions to the received data.
        rx_payloads.iter_mut().for_each(|payload| {
            do_actions(payload, &self.content);
        });

        for target_class in self.models.actor.target_classes.clone().iter() {
            self.talk_to_class(target_class, &rx_payloads, bucket);
        }
        self.talk_to_class(&self.node_info.node_class.clone(), &rx_payloads, bucket);

        bucket
            .devices
            .entry(self.node_info.id)
            .and_modify(|device| *device = self.clone())
            .or_insert(self.clone());
    }

    fn sidelink_stage(&mut self, bucket: &mut DeviceBucket) {
        // Receive data from the uplink.
        self.receive(bucket);
        // Receive data from the peers.
        self.receive_sl(bucket);
        // Store any data that needs to be forwarded in the next step.
        // Respond to the peers.
        bucket
            .devices
            .entry(self.node_info.id)
            .and_modify(|device| *device = self.clone())
            .or_insert(self.clone());
    }

    fn downlink_stage(&mut self, bucket: &mut DeviceBucket) {
        debug!(
            "Downlink stage for node: {} id at step: {}",
            self.node_info.id, self.step
        );
        let response = bucket.data_lake.response_for(self.node_info.id);
        self.respond(response, bucket);

        if self.step == self.models.power.peek_time_to_off() {
            self.power_state = PowerState::Off;
            if self.models.power.has_next_time_to_on() {
                bucket.add_to_schedule(self.node_info.id);
            }
            bucket.stop_node(self.node_info.id);
            bucket
                .devices
                .entry(self.node_info.id)
                .and_modify(|device| *device = self.clone())
                .or_insert(self.clone());
        }
        bucket
            .devices
            .entry(self.node_info.id)
            .and_modify(|device| *device = self.clone())
            .or_insert(self.clone());
    }
}
