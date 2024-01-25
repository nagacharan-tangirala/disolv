use crate::bucket::DeviceBucket;
use log::{debug, warn};
use pavenet_core::entity::{NodeClass, NodeInfo, NodeOrder};
use pavenet_core::message::{DPayload, NodeContent, PayloadInfo, TxStatus};
use pavenet_core::message::{DResponse, DataSource, TxMetrics};
use pavenet_core::mobility::MapState;
use pavenet_core::power::{PowerManager, PowerState};
use pavenet_core::radio::{DActions, DLink, LinkProperties, OutgoingStats};
use pavenet_engine::bucket::TimeMS;
use pavenet_engine::entity::{Entity, Movable, Schedulable, Tiered};
use pavenet_engine::node::GNode;
use pavenet_engine::radio::{Receiver, Responder, Transmitter};
use pavenet_models::actions::{do_actions, filter_blobs_to_fwd, set_actions_before_tx};
use pavenet_models::actor::Actor;
use pavenet_models::compose::Composer;
use pavenet_models::flow::FlowRegister;
use pavenet_models::reply::Replier;
use pavenet_models::select::Selector;
use typed_builder::TypedBuilder;

pub type TNode = GNode<DeviceBucket, Device, NodeOrder>;

#[derive(Debug, Clone, TypedBuilder)]
pub struct DeviceModel {
    pub power: PowerManager,
    pub flow: FlowRegister,
    pub sl_flow: FlowRegister,
    pub composer: Composer,
    pub replier: Replier,
    pub actor: Actor,
    pub selector: Vec<(NodeClass, Selector)>,
}

impl DeviceModel {
    fn select_links(
        &self,
        link_options: Vec<DLink>,
        target_class: &NodeClass,
        stats: &Vec<Option<&OutgoingStats>>,
    ) -> Option<Vec<DLink>> {
        for selectors in self.selector.iter() {
            if selectors.0 == *target_class {
                return Some(selectors.1.do_selection(link_options, stats));
            }
        }
        None
    }
}

#[derive(Clone, Debug, TypedBuilder)]
pub struct Device {
    pub node_info: NodeInfo,
    pub models: DeviceModel,
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

    fn talk_to_class(
        &mut self,
        target_class: &NodeClass,
        rx_payloads: &Vec<DPayload>,
        bucket: &mut DeviceBucket,
    ) {
        let link_options = match bucket.link_options_for(
            self.node_info.id,
            &self.node_info.node_type,
            target_class,
        ) {
            Some(links) => links,
            None => return,
        };
        debug!(
            "Node {} received {} payloads at step {}",
            self.node_info.id,
            rx_payloads.len(),
            self.step
        );

        let stats = bucket.stats_for(&link_options);
        let targets = match self.models.select_links(link_options, target_class, &stats) {
            Some(links) => links,
            None => return,
        };

        let payload = self
            .models
            .composer
            .compose_payload(target_class, self.content);

        targets.into_iter().for_each(|target_link| {
            if let Some(target_node) = bucket.node_of(target_link.target) {
                let mut this_payload = payload.clone();
                match rx_payloads {
                    Some(ref payloads) => {
                        let mut blobs = filter_blobs_to_fwd(&target_node.content, payloads);
                        self.models
                            .composer
                            .append_blobs_to(&mut this_payload, &mut blobs);
                    }
                    None => (),
                }
                let actions = self.models.actor.actions_for(target_class);
                let prepared_payload = set_actions_before_tx(this_payload, actions);
                self.transmit(prepared_payload, target_link, bucket);
            } else {
                warn!("Missing node ID {} ", target_link.target);
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

    fn transmit(&mut self, payload: DPayload, target_link: DLink, bucket: &mut DeviceBucket) {
        debug!(
            "Transmitting payload from node {} to node {} with blobs {}",
            payload.node_state.node_info.id.as_u32(),
            target_link.target.as_u32(),
            payload.metadata.data_blobs.len()
        );

        self.models.flow.register_outgoing_attempt(&payload);
        let tx_metrics = bucket.network.transfer(&payload);
        bucket
            .resultant
            .add_tx_data(self.step, &target_link, &payload, tx_metrics);

        if tx_metrics.tx_status == TxStatus::Ok {
            self.models.flow.register_outgoing_feasible(&payload);
            bucket.data_lake.add_payload_to(target_link.target, payload);
        }
    }

    fn transmit_sl(&mut self, payload: DPayload, target_link: DLink, bucket: &mut DeviceBucket) {
        debug!(
            "Transmitting payload from node {} to node {}",
            payload.node_state.node_info.id.as_u32(),
            target_link.target.as_u32()
        );

        self.models.sl_flow.register_outgoing_attempt(&payload);
        let sl_metrics = bucket.network.transfer(&payload);
        bucket
            .resultant
            .add_tx_data(self.step, &target_link, &payload, sl_metrics);

        if sl_metrics.tx_status == TxStatus::Ok {
            self.models.sl_flow.register_outgoing_feasible(&payload);
            bucket
                .data_lake
                .add_sl_payload_to(target_link.target, payload);
        }
    }
}

impl Receiver<DeviceBucket, PayloadInfo, NodeContent> for Device {
    type C = NodeClass;

    fn receive(&mut self, bucket: &mut DeviceBucket) -> Option<Vec<DPayload>> {
        bucket.data_lake.payloads_for(self.node_info.id)
    }

    fn receive_sl(&mut self, bucket: &mut DeviceBucket) -> Option<Vec<DPayload>> {
        bucket.data_lake.sl_payloads_for(self.node_info.id)
    }
}

impl Responder<DeviceBucket, DataSource, TxMetrics> for Device {
    fn respond(&mut self, response: Option<DResponse>, bucket: &mut DeviceBucket) {
        // send to talk_to_peers
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
        let mut rx_payloads = self.receive(bucket);
        if let Some(ref mut payloads) = rx_payloads {
            self.models.flow.register_incoming(payloads);
            payloads.iter_mut().for_each(|payload| {
                do_actions(payload, &self.content);
            });
            debug!(
                "Node {} received {} payloads at step {}",
                self.node_info.id,
                payloads.len(),
                self.step
            );
        }

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
