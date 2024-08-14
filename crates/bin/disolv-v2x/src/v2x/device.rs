use std::fmt::Debug;

use log::debug;
use typed_builder::TypedBuilder;

use disolv_core::agent::{
    Activatable, Agent, AgentClass, AgentKind, AgentProperties, Movable, Orderable,
};
use disolv_core::agent::{AgentId, AgentOrder};
use disolv_core::bucket::TimeMS;
use disolv_core::metrics::Measurable;
use disolv_core::metrics::Resource;
use disolv_core::radio::{Link, Receiver, Transmitter};
use disolv_models::device::actions::{
    complete_actions, filter_units_to_fwd, set_actions_before_tx,
};
use disolv_models::device::actor::Actor;
use disolv_models::device::flow::FlowRegister;
use disolv_models::device::mobility::MapState;
use disolv_models::device::models::{Compose, LinkSelector};
use disolv_models::device::power::{PowerManager, PowerState};
use disolv_models::net::network::NetworkSlice;
use disolv_models::net::radio::{CommStats, LinkProperties};

use crate::models::compose::Composer;
use crate::models::message::{DataBlob, DataType, MessageType, PayloadInfo, TxStatus, V2XPayload};
use crate::models::network::V2XSlice;
use crate::models::select::Selector;
use crate::v2x::bucket::DeviceBucket;

#[derive(Clone, Copy, Debug, Default, TypedBuilder)]
pub struct DeviceInfo {
    pub id: AgentId,
    pub device_type: AgentKind,
    pub device_class: AgentClass,
    pub agent_order: AgentOrder,
}

impl AgentProperties for DeviceInfo {
    fn id(&self) -> AgentId {
        self.id
    }

    fn kind(&self) -> &AgentKind {
        &self.device_type
    }

    fn class(&self) -> &AgentClass {
        &self.device_class
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct DeviceModel {
    pub power: PowerManager,
    pub flow: FlowRegister,
    pub sl_flow: FlowRegister,
    pub composer: Composer,
    pub actor: Actor<DataType>,
    pub selector: Vec<(AgentClass, Selector)>,
}

impl DeviceModel {
    fn select_links(
        &self,
        link_options: Vec<Link<LinkProperties>>,
        target_class: &AgentClass,
        stats: &Vec<&CommStats>,
    ) -> Option<Vec<Link<LinkProperties>>> {
        for selectors in self.selector.iter() {
            if selectors.0 == *target_class {
                return Some(selectors.1.select_link(link_options, stats));
            }
        }
        None
    }
}

#[derive(Clone, Debug, TypedBuilder)]
pub struct Device {
    pub device_info: DeviceInfo,
    pub models: DeviceModel,
    #[builder(default)]
    pub step: TimeMS,
    #[builder(default)]
    pub power_state: PowerState,
    #[builder(default)]
    pub map_state: MapState,
    #[builder(default)]
    pub stats: CommStats,
}

impl Device {
    fn talk_to_class(
        &mut self,
        target_class: &AgentClass,
        rx_payloads: &Option<Vec<V2XPayload>>,
        bucket: &mut DeviceBucket,
    ) {
        let link_options = match bucket.link_options_for(
            self.device_info.id,
            &self.device_info.device_type,
            target_class,
        ) {
            Some(links) => links,
            None => return,
        };

        let mut stats: Vec<&CommStats> = Vec::with_capacity(link_options.len());
        link_options.iter().for_each(|link| {
            let link_stats = bucket.stats_for(&link.target);
            if link_stats.is_some() {
                stats.push(link_stats.unwrap());
            }
        });

        let targets = match self.models.select_links(link_options, target_class, &stats) {
            Some(links) => links,
            None => return,
        };

        let payload = self
            .models
            .composer
            .compose(target_class, &self.device_info);

        targets.into_iter().for_each(|target_link| {
            let mut this_payload = payload.clone();

            // If we know about the target agent, take payload forwarding decisions.
            if let Some(target_state) = bucket.device_info_of(&target_link.target) {
                match rx_payloads {
                    Some(ref payloads) => {
                        let mut blobs = filter_units_to_fwd(target_state, payloads);
                        self.models
                            .composer
                            .append_blobs_to(&mut this_payload, &mut blobs);
                    }
                    None => (),
                }
            }

            let actions = self.models.actor.actions_for(target_class);
            let prepared_payload = set_actions_before_tx(this_payload, actions);
            if target_class == &self.device_info.device_class {
                self.transmit_sl(prepared_payload, target_link, bucket);
            } else {
                self.transmit(prepared_payload, target_link, bucket);
            }
        });
    }
}

impl Movable<DeviceBucket> for Device {
    type M = MapState;

    fn mobility(&self) -> &MapState {
        &self.map_state
    }

    fn set_mobility(&mut self, bucket: &mut DeviceBucket) {
        self.map_state = bucket
            .positions_for(self.device_info.id, &self.device_info.device_type)
            .unwrap_or(self.map_state);
        bucket.models.output.basic_results.positions.add_data(
            self.step,
            self.device_info.id,
            &self.map_state,
        );
    }
}

impl Activatable<DeviceBucket> for Device {
    fn activate(&mut self, bucket: &mut DeviceBucket) {
        debug!("Starting agent: {}", self.device_info.id);
        self.power_state = PowerState::On;
        bucket.update_device_info_of(self.device_info.id, self.device_info);
    }

    fn deactivate(&mut self) {
        debug!("Stopping agent: {}", self.device_info.id);
        self.power_state = PowerState::On;
    }

    fn is_deactivated(&self) -> bool {
        self.power_state == PowerState::Off
    }

    fn has_activation(&self) -> bool {
        self.models.power.has_next_time_to_on()
    }

    fn time_of_activation(&mut self) -> TimeMS {
        self.models.power.pop_time_to_on()
    }
}

impl Orderable for Device {
    fn order(&self) -> AgentOrder {
        self.device_info.agent_order
    }
}

impl
    Transmitter<
        DeviceBucket,
        DataType,
        DataBlob,
        LinkProperties,
        PayloadInfo,
        DeviceInfo,
        MessageType,
    > for Device
{
    fn transmit(
        &mut self,
        payload: V2XPayload,
        target_link: Link<LinkProperties>,
        bucket: &mut DeviceBucket,
    ) {
        debug!(
            "Transmitting payload from agent {} to agent {} with blobs {}",
            payload.agent_state.id(),
            target_link.target,
            payload.data_units.len()
        );

        self.models.flow.register_outgoing_attempt(&payload);
        let tx_metrics = bucket.models.network.transfer(&payload);
        match &mut bucket.models.output.tx_data_writer {
            Some(tx) => tx.add_data(self.step, &target_link, &payload, tx_metrics),
            None => {}
        }

        if tx_metrics.tx_status == TxStatus::Ok {
            self.models.flow.register_outgoing_feasible(&payload);
            bucket
                .models
                .data_lake
                .add_payload_to(target_link.target, payload);
        }
    }

    fn transmit_sl(
        &mut self,
        payload: V2XPayload,
        target_link: Link<LinkProperties>,
        bucket: &mut DeviceBucket,
    ) {
        debug!(
            "Transmitting SL payload from agent {} to agent {}",
            payload.agent_state.id(),
            target_link.target
        );

        self.models.sl_flow.register_outgoing_attempt(&payload);
        let sl_metrics = bucket.models.network.transfer(&payload);

        match &mut bucket.models.output.tx_data_writer {
            Some(tx) => tx.add_data(self.step, &target_link, &payload, sl_metrics),
            None => {}
        }

        if sl_metrics.tx_status == TxStatus::Ok {
            self.models.sl_flow.register_outgoing_feasible(&payload);
            bucket
                .models
                .data_lake
                .add_sl_payload_to(target_link.target, payload);
        }
    }
}

impl
    Receiver<DeviceBucket, DataType, DataBlob, LinkProperties, PayloadInfo, DeviceInfo, MessageType>
    for Device
{
    fn receive(&mut self, bucket: &mut DeviceBucket) -> Option<Vec<V2XPayload>> {
        bucket.models.data_lake.payloads_for(self.device_info.id)
    }

    fn receive_sl(&mut self, bucket: &mut DeviceBucket) -> Option<Vec<V2XPayload>> {
        bucket.models.data_lake.sl_payloads_for(self.device_info.id)
    }
}

impl Agent<DeviceBucket> for Device {
    type P = DeviceInfo;

    fn id(&self) -> AgentId {
        self.device_info.id
    }

    fn stage_one(&mut self, bucket: &mut DeviceBucket) {
        self.step = bucket.step;
        self.set_mobility(bucket);
        bucket.update_stats_of(self.device_info.id, self.stats);
        debug!(
            "Uplink stage for agent: {} id at step: {}",
            self.device_info.id, self.step
        );
        self.models.flow.reset();

        // Receive data from the downstream agents.
        let mut rx_payloads = self.receive(bucket);
        if let Some(ref mut payloads) = rx_payloads {
            self.models.flow.register_incoming(payloads);
            payloads.iter_mut().for_each(|payload| {
                complete_actions(payload, &self.device_info);
            });
        }

        for target_class in self.models.actor.target_classes.clone().iter() {
            self.talk_to_class(target_class, &rx_payloads, bucket);
        }
        self.talk_to_class(&self.device_info.device_class.clone(), &rx_payloads, bucket);
    }

    fn stage_two_reverse(&mut self, _bucket: &mut DeviceBucket) {}

    fn stage_three(&mut self, bucket: &mut DeviceBucket) {
        // Receive data from the peers.
        self.receive_sl(bucket);
    }

    fn stage_four_reverse(&mut self, bucket: &mut DeviceBucket) {
        debug!(
            "Downlink stage for agent: {} id at step: {}",
            self.device_info.id, self.step
        );

        bucket.models.output.basic_results.rx_counts.add_data(
            self.step,
            self.device_info.id,
            &self.models.flow.comm_stats.outgoing_stats,
        );

        if self.step == self.models.power.peek_time_to_off() {
            self.power_state = PowerState::Off;
        }
    }

    fn stage_five(&mut self, _bucket: &mut DeviceBucket) {
        self.stats = self.models.flow.comm_stats;
    }
}
