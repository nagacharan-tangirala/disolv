use crate::bucket::DeviceBucket;
use disolv_core::agent::{Activatable, Agent, Movable, Orderable};
use disolv_core::agent::{AgentId, AgentOrder};
use disolv_core::bucket::TimeMS;
use disolv_core::core::Core;
use disolv_core::radio::{Receiver, Responder, Transmitter};
use disolv_models::bucket::flow::FlowRegister;
use disolv_models::device::actions::{do_actions, filter_blobs_to_fwd, set_actions_before_tx};
use disolv_models::device::actor::Actor;
use disolv_models::device::compose::Composer;
use disolv_models::device::mobility::MapState;
use disolv_models::device::power::{PowerManager, PowerState};
use disolv_models::device::reply::Replier;
use disolv_models::device::select::Selector;
use disolv_models::device::types::{DeviceClass, DeviceInfo, DeviceStats};
use disolv_models::net::message::{DPayload, DeviceContent, PayloadInfo, TxStatus};
use disolv_models::net::message::{DResponse, DataSource, TxMetrics};
use disolv_models::net::radio::{DLink, LinkProperties};
use log::debug;
use std::fmt::Debug;
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, TypedBuilder)]
pub struct DeviceModel {
    pub power: PowerManager,
    pub flow: FlowRegister,
    pub sl_flow: FlowRegister,
    pub composer: Composer,
    pub replier: Replier,
    pub actor: Actor,
    pub selector: Vec<(DeviceClass, Selector)>,
}

impl DeviceModel {
    fn select_links(
        &self,
        link_options: Vec<DLink>,
        target_class: &DeviceClass,
        stats: &Vec<&DeviceStats>,
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
    pub device_info: DeviceInfo,
    pub models: DeviceModel,
    #[builder(default)]
    pub step: TimeMS,
    #[builder(default)]
    pub power_state: PowerState,
    #[builder(default)]
    pub map_state: MapState,
    #[builder(default)]
    pub content: DeviceContent,
    #[builder(default)]
    pub stats: DeviceStats,
}

impl Device {
    fn compose_content(&self) -> DeviceContent {
        DeviceContent {
            device_info: self.device_info,
            map_state: self.map_state,
        }
    }

    fn compute_stats(&mut self) {
        self.stats = DeviceStats::builder()
            .incoming_stats(self.models.flow.in_stats)
            .outgoing_stats(self.models.flow.out_stats)
            .device_content(self.content)
            .build();
    }

    fn talk_to_class(
        &mut self,
        target_class: &DeviceClass,
        rx_payloads: &Option<Vec<DPayload>>,
        core: &mut Core<Self, DeviceBucket>,
    ) {
        let link_options = match core.bucket.link_options_for(
            self.device_info.id,
            &self.device_info.device_type,
            target_class,
        ) {
            Some(links) => links,
            None => return,
        };

        let stats: Vec<&DeviceStats> = link_options
            .iter()
            .map(|link| core.stats_of(&link.target))
            .collect();

        let targets = match self.models.select_links(link_options, target_class, &stats) {
            Some(links) => links,
            None => return,
        };

        let payload = self
            .models
            .composer
            .compose_payload(target_class, self.content);

        targets.into_iter().for_each(|target_link| {
            let target_stats = core.stats_of(&target_link.target);
            let mut this_payload = payload.clone();
            match rx_payloads {
                Some(ref payloads) => {
                    let mut blobs = filter_blobs_to_fwd(&target_stats.device_content, payloads);
                    self.models
                        .composer
                        .append_blobs_to(&mut this_payload, &mut blobs);
                }
                None => (),
            }
            let actions = self.models.actor.actions_for(target_class);
            let prepared_payload = set_actions_before_tx(this_payload, actions);
            if target_class == &self.device_info.device_class {
                self.transmit_sl(prepared_payload, target_link, &mut core.bucket);
            } else {
                self.transmit(prepared_payload, target_link, &mut core.bucket);
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
        bucket
            .models
            .result_writer
            .add_agent_pos(self.step, self.device_info.id, &self.map_state);
    }
}

impl Activatable for Device {
    fn activate(&mut self) {
        debug!("Starting agent: {}", self.device_info.id);
        self.power_state = PowerState::On;
    }

    fn deactivate(&mut self) {
        debug!("Stopping agent: {}", self.device_info.id);
        self.power_state = PowerState::On;
    }

    fn is_deactivated(&self) -> bool {
        self.power_state == PowerState::Off
    }

    fn time_to_activation(&mut self) -> TimeMS {
        self.models.power.pop_time_to_on()
    }
}

impl Orderable for Device {
    fn order(&self) -> AgentOrder {
        self.device_info.agent_order
    }
}

impl Transmitter<DeviceContent, DeviceBucket, LinkProperties, PayloadInfo> for Device {
    type AgentClass = DeviceClass;

    fn transmit(&mut self, payload: DPayload, target_link: DLink, bucket: &mut DeviceBucket) {
        debug!(
            "Transmitting payload from agent {} to agent {} with blobs {}",
            payload.agent_state.device_info.id.as_u32(),
            target_link.target.as_u32(),
            payload.metadata.data_blobs.len()
        );

        self.models.flow.register_outgoing_attempt(&payload);
        let tx_metrics = bucket.models.network.transfer(&payload);
        bucket
            .models
            .result_writer
            .add_tx_data(self.step, &target_link, &payload, tx_metrics);

        if tx_metrics.tx_status == TxStatus::Ok {
            self.models.flow.register_outgoing_feasible(&payload);
            bucket
                .models
                .data_lake
                .add_payload_to(target_link.target, payload);
        }
    }

    fn transmit_sl(&mut self, payload: DPayload, target_link: DLink, bucket: &mut DeviceBucket) {
        debug!(
            "Transmitting SL payload from agent {} to agent {}",
            payload.agent_state.device_info.id.as_u32(),
            target_link.target.as_u32()
        );

        self.models.sl_flow.register_outgoing_attempt(&payload);
        let sl_metrics = bucket.models.network.transfer(&payload);
        bucket
            .models
            .result_writer
            .add_tx_data(self.step, &target_link, &payload, sl_metrics);

        if sl_metrics.tx_status == TxStatus::Ok {
            self.models.sl_flow.register_outgoing_feasible(&payload);
            bucket
                .models
                .data_lake
                .add_sl_payload_to(target_link.target, payload);
        }
    }
}

impl Receiver<DeviceContent, DeviceBucket, PayloadInfo> for Device {
    type C = DeviceClass;

    fn receive(&mut self, bucket: &mut DeviceBucket) -> Option<Vec<DPayload>> {
        bucket.models.data_lake.payloads_for(self.device_info.id)
    }

    fn receive_sl(&mut self, bucket: &mut DeviceBucket) -> Option<Vec<DPayload>> {
        bucket.models.data_lake.sl_payloads_for(self.device_info.id)
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

impl Agent<DeviceBucket> for Device {
    type AS = DeviceStats;

    fn id(&self) -> AgentId {
        self.device_info.id
    }

    fn stats(&self) -> Self::AS {
        self.stats
    }

    fn stage_one(&mut self, core: &mut Core<Self, DeviceBucket>) {
        self.step = core.bucket.step;
        let bucket = &mut core.bucket;
        self.set_mobility(bucket);
        self.content = self.compose_content();

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
                do_actions(payload, &self.content);
            });
        }

        for target_class in self.models.actor.target_classes.clone().iter() {
            self.talk_to_class(target_class, &rx_payloads, core);
        }
        self.talk_to_class(&self.device_info.device_class.clone(), &rx_payloads, core);
    }

    fn stage_two_reverse(&mut self, _core: &mut Core<Self, DeviceBucket>) {}

    fn stage_three(&mut self, core: &mut Core<Self, DeviceBucket>) {
        // Receive data from the peers.
        self.receive_sl(&mut core.bucket);
    }

    fn stage_four_reverse(&mut self, core: &mut Core<Self, DeviceBucket>) {
        debug!(
            "Downlink stage for agent: {} id at step: {}",
            self.device_info.id, self.step
        );
        let bucket = &mut core.bucket;
        let response = bucket.models.data_lake.response_for(self.device_info.id);
        self.respond(response, bucket);

        core.bucket.models.result_writer.add_rx_counts(
            self.step,
            self.device_info.id,
            &self.models.flow.out_stats,
        );

        if self.step == self.models.power.peek_time_to_off() {
            self.power_state = PowerState::Off;
            if self.models.power.has_next_time_to_on() {
                core.add_agent(self.device_info.id, self.models.power.pop_time_to_on());
            }
        }
    }

    fn stage_five(&mut self, _core: &mut Core<Self, DeviceBucket>) {
        self.compute_stats();
    }
}
