use crate::bucket::DeviceBucket;
use crate::d_model::DeviceModel;
use log::{debug, error};
use pavenet_core::entity::{NodeClass, NodeInfo, NodeOrder, NodeType};
use pavenet_core::message::{DPayload, DataType, NodeContent, PayloadInfo, TransferStatus};
use pavenet_core::message::{DResponse, DataSource, TransferMetrics};
use pavenet_core::mobility::MapState;
use pavenet_core::power::PowerState;
use pavenet_core::radio::ActionType::Consume;
use pavenet_core::radio::{DLink, LinkProperties};
use pavenet_engine::bucket::TimeS;
use pavenet_engine::entity::{Entity, Movable, Schedulable, Tiered};
use pavenet_engine::message::Responder;
use pavenet_engine::message::Transmitter;
use pavenet_engine::node::GNode;
use pavenet_engine::radio::{GLink, LinkFeatures, OutgoingStats, RxChannel, TxChannel};
use typed_builder::TypedBuilder;

pub type TNode = GNode<DeviceBucket, Device, NodeOrder>;

#[derive(Clone, Debug, TypedBuilder)]
pub struct Device {
    pub node_info: NodeInfo,
    pub models: DeviceModel,
    #[builder(default)]
    pub target_classes: Vec<NodeClass>,
    #[builder(default)]
    pub step: TimeS,
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

    fn assign_forwarding(
        &mut self,
        device_bucket: &DeviceBucket,
        payload: &mut DPayload,
        link: &DLink,
        target_class: &NodeClass,
        to_forward: &mut Vec<DPayload>,
    ) {
        to_forward.into_iter().for_each(|each_payload| {
            for blob in each_payload.metadata.data_blobs.into_iter() {
                if blob.action.action_type == Consume {
                    error!("This should have been consumed by now");
                    panic!("consume payload appears to be forwarded");
                }

                if let Some(target_id) = blob.action.to_node {
                    if target_id == link.target {
                        payload.metadata.data_blobs.push(blob);
                    }
                    continue;
                }

                if let Some(class) = blob.action.to_class {
                    if class == *target_class {
                        payload.metadata.data_blobs.push(blob);
                    }
                    continue;
                }

                if let Some(target_kind) = blob.action.to_kind {
                    match device_bucket.node_of(link.target) {
                        Some(node) => {
                            if node.node_info.node_type == target_kind {
                                payload.metadata.data_blobs.push(blob);
                            }
                        }
                        None => {}
                    };
                }
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
        self.power_state = PowerState::Off;
    }

    fn is_stopped(&self) -> bool {
        self.power_state == PowerState::Off
    }

    fn time_to_add(&mut self) -> TimeS {
        debug!(
            "Node {} will be added at {}",
            self.node_info.id,
            self.models.power.pop_time_to_on()
        );
        self.models.power.pop_time_to_on()
    }
}

impl Transmitter<DeviceBucket, LinkProperties, NodeContent, PayloadInfo> for Device {
    type NodeClass = NodeClass;

    fn collect(&mut self, bucket: &mut DeviceBucket) -> Vec<DPayload> {
        let incoming = match bucket.data_lake.payloads_for(self.node_info.id) {
            Some(incoming) => incoming,
            None => return Vec::new(),
        };
        let feasible = self.models.radio.complete_transfers(incoming);
        let to_forward = self.models.radio.perform_actions(&self.node_info, feasible);
        return to_forward;
    }

    fn find_target(&mut self, bucket: &mut DeviceBucket) -> Option<DLink> {
        let link_options =
            match bucket.link_options_for(self.node_info.id, &self.node_info.node_type) {
                Some(links) => links,
                None => return None,
            };
        let stats = bucket.stats_for(&link_options);
        let target_link = match self.models.selector {
            Some(ref mut selector) => selector.select_target(link_options, &stats),
            None => return None,
        };
        return Some(target_link);
    }

    fn compose(&mut self, target_class: &NodeClass) -> Option<DPayload> {
        let node_content = self.compose_content();
        return match self.models.composer {
            Some(ref mut composer) => {
                let device_payload = composer.compose_payload(target_class, node_content);
                Some(device_payload)
            }
            None => None,
        };
    }

    fn transmit(&mut self, payload: DPayload, link: DLink, bucket: &mut DeviceBucket) {
        debug!(
            "Transmitting from {} to {} at {}",
            self.node_info.id, target_link.target, self.step
        );
        self.models.radio.out_stats.update(&payload.metadata);
        bucket
            .resultant
            .result_writer
            .add_tx_data(self.step, &payload);
        bucket.data_lake.add_payload_to(link.target, payload)
    }
}

impl Responder<DeviceBucket, DataSource, TransferStatus> for Device {
    fn receive(&mut self, bucket: &mut DeviceBucket) -> Option<DResponse> {
        let response = match bucket.data_lake.response_for(self.node_info.id) {
            Some(response) => response,
            None => return None,
        };
        match self.models.composer {
            Some(ref mut composer) => {
                composer.update_sources(response.content.as_ref().unwrap());
            }
            None => return None,
        };
        Some(response)
    }

    fn respond(&mut self, response: Option<DResponse>, bucket: &mut DeviceBucket) {
        for (node_id, transfer_stats) in self.models.radio.transfer_stats().into_iter() {
            let this_response = match self.models.responder {
                Some(ref mut responder) => responder.compose_response(response, transfer_stats),
                None => return,
            };
            bucket.data_lake.add_response_to(node_id, this_response);
        }
    }
}

impl Entity<DeviceBucket, NodeClass> for Device {
    fn uplink_stage(&mut self, bucket: &mut DeviceBucket) {
        self.step = bucket.step;
        self.power_state = PowerState::On;
        self.set_mobility(bucket);

        let mut payloads_to_fwd = self.collect(bucket);
        self.target_classes.iter().for_each(|target_class| {
            match self.find_target(bucket) {
                Some(link) => {
                    match self.compose(target_class) {
                        Some(mut payload) => {
                            self.assign_forwarding(
                                bucket,
                                &mut payload,
                                &link,
                                target_class,
                                &mut payloads_to_fwd,
                            );

                            self.transmit(payload, link, bucket);
                        }
                        None => {}
                    };
                }
                None => {}
            };
        });

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
            bucket.add_to_schedule(self.node_info.id);
            bucket.stop_node(self.node_info.id);
        }
        self.models.radio.reset();
    }
}
