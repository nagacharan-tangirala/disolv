use crate::bucket::DeviceBucket;
use crate::d_model::DeviceModel;
use crate::models::power::PowerState;
use pavenet_core::link::SelectedLink;
use pavenet_core::mobility::MapState;
use pavenet_core::node_info::id::NodeId;
use pavenet_core::node_info::kind::NodeType;
use pavenet_core::node_info::order::Order;
use pavenet_core::node_info::NodeInfo;
use pavenet_core::payload::NodeContent;
use pavenet_core::times::ts::TimeS;
use pavenet_engine::entity::{Entity, Movable, Schedulable, Tiered};
use pavenet_engine::node::Node;
use pavenet_engine::payload::{Payload, PayloadContent, Transmitter};
use pavenet_engine::response::{Responder, Response};
use typed_builder::TypedBuilder;

pub type TNode = Node<DeviceBucket, Device, NodeId, NodeType, Order, TimeS>;

#[derive(Clone, Debug, TypedBuilder)]
pub struct Device {
    pub node_info: NodeInfo,
    pub models: DeviceModel,
    pub map_state: MapState,
    pub power_state: PowerState,
    pub link_targets: Vec<NodeType>,
    pub step: TimeS,
}

impl Device {
    fn compose_content(&self) -> NodeContent {
        NodeContent {
            node_info: self.node_info,
            map_state: self.map_state,
        }
    }

    fn target_link(
        &mut self,
        bucket: &mut DeviceBucket,
        target: &NodeType,
    ) -> Option<SelectedLink> {
        let links = match bucket.links_for(self.node_info.id, target) {
            Some(links) => links,
            None => return None,
        };
        let stats = bucket.stats_for(&links);
        let target_link = self.models.selector.select_target(links, &stats);
        Some(target_link)
    }
}

impl Tiered<Order> for Device {
    fn tier(&self) -> &Order {
        &self.node_info.order
    }

    fn set_tier(&mut self, tier: Order) {
        self.node_info.order = tier;
    }
}

impl Movable<MapState> for Device {
    fn mobility(&self) -> &MapState {
        &self.map_state
    }

    fn set_mobility(&mut self, mobility_info: MapState) {
        self.map_state = mobility_info;
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

impl Transmitter<DeviceBucket, TimeS> for Device {
    fn transmit(&mut self, bucket: &mut DeviceBucket) {
        let incoming_data = bucket.data_lake.payloads_for(self.node_info.id);
        let payloads_to_fwd = self.models.channel.payloads_to_forward(incoming_data);

        for target in self.link_targets.iter() {
            let target_link = match self.target_link(bucket, target) {
                Some(value) => value,
                None => continue,
            };
            let mut payload = self.models.composer.compose_payload(
                target,
                self.compose_content(),
                &payloads_to_fwd,
            );
            payload.metadata.selected_link = target_link;
            bucket
                .data_lake
                .add_payload_to(target_link.node_id, payload);
        }
    }
}

impl Responder<DeviceBucket, TimeS> for Device {
    fn respond(&mut self, bucket: &mut DeviceBucket) {
        todo!()
    }
}

impl Entity<DeviceBucket, Order, TimeS> for Device {
    fn uplink_stage(&mut self, bucket: &mut DeviceBucket) {
        self.step = bucket.step;
        match bucket.positions_for(self.node_info.id, &self.node_info.node_type) {
            Some(mobility) => self.set_mobility(mobility),
            None => {}
        };

        self.transmit(bucket);

        if self.step == self.models.power.peek_time_to_off() {
            bucket.add_to_pop_now(self.node_info.id);
            bucket.schedule_future_add(self.node_info.id, self.models.power.pop_time_to_on());
        }
        bucket
            .devices
            .entry(self.node_info.id)
            .or_insert(self.clone());
    }

    fn downlink_stage(&mut self, bucket: &mut DeviceBucket) {}
}
