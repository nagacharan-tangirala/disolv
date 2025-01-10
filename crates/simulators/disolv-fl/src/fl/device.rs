use burn::tensor::backend::AutodiffBackend;
use hashbrown::HashMap;
use log::{debug, trace};
use typed_builder::TypedBuilder;

use disolv_core::agent::{
    Activatable, Agent, AgentClass, AgentId, AgentKind, AgentOrder, AgentProperties, Movable,
    Orderable,
};
use disolv_core::bucket::TimeMS;
use disolv_core::metrics::Bytes;
use disolv_core::radio::{Action, Link, Receiver, Transmitter};
use disolv_models::device::actions::{
    complete_actions, filter_units_to_fwd, set_actions_before_tx,
};
use disolv_models::device::actor::Actor;
use disolv_models::device::directions::Directions;
use disolv_models::device::flow::FlowRegister;
use disolv_models::device::metrics::MegaHertz;
use disolv_models::device::mobility::MapState;
use disolv_models::device::models::LinkSelect;
use disolv_models::device::power::{PowerManager, PowerState};
use disolv_models::net::metrics::Bandwidth;
use disolv_models::net::radio::{CommStats, LinkProperties};
use disolv_output::tables::payload::PayloadUpdate;
use disolv_output::tables::tx::TxData;

use crate::fl::agent::FAgent;
use crate::fl::bucket::FlBucket;
use crate::models::ai::compose::FlComposer;
use crate::models::device::energy::EnergyType;
use crate::models::device::hardware::Hardware;
use crate::models::device::link::LinkSelector;
use crate::models::device::message::{FlPayload, FlPayloadInfo, Message, MessageType, MessageUnit};

#[derive(Clone, Copy, Debug, Default, TypedBuilder)]
pub(crate) struct DeviceInfo {
    pub(crate) id: AgentId,
    pub(crate) agent_type: AgentKind,
    pub(crate) agent_class: AgentClass,
    pub(crate) agent_order: AgentOrder,
    #[builder(default)]
    pub(crate) cpu: MegaHertz,
    #[builder(default)]
    pub(crate) memory: Bytes,
    #[builder(default)]
    pub(crate) bandwidth: Bandwidth,
}

impl AgentProperties for DeviceInfo {
    fn id(&self) -> AgentId {
        self.id
    }

    fn kind(&self) -> &AgentKind {
        &self.agent_type
    }

    fn class(&self) -> &AgentClass {
        &self.agent_class
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct DeviceModels {
    pub power: PowerManager,
    pub flow: FlowRegister,
    pub sl_flow: FlowRegister,
    pub composer: FlComposer,
    pub actor: Actor<MessageType>,
    pub directions: Directions,
    pub hardware: Hardware,
    pub energy: EnergyType,
    pub link_selector: Vec<(AgentClass, LinkSelector)>,
}

#[derive(Clone, TypedBuilder)]
pub(crate) struct Device<B: AutodiffBackend> {
    pub(crate) fl_agent: FAgent<B>,
    pub(crate) device_info: DeviceInfo,
    pub(crate) models: DeviceModels,
    #[builder(default)]
    pub(crate) step: TimeMS,
    #[builder(default)]
    pub(crate) power_state: PowerState,
    #[builder(default)]
    pub(crate) map_state: MapState,
    #[builder(default)]
    pub(crate) stats: CommStats,
}

impl<B: AutodiffBackend> Device<B> {
    fn select_links(
        &self,
        link_options: Vec<Link<LinkProperties>>,
        target_class: &AgentClass,
        stats: &Vec<&CommStats>,
    ) -> Option<Vec<Link<LinkProperties>>> {
        for selectors in self.models.link_selector.iter() {
            if selectors.0 == *target_class {
                return Some(selectors.1.select_link(link_options, stats));
            }
        }
        None
    }

    fn get_links(
        &self,
        target_class: &AgentClass,
        bucket: &mut FlBucket<B>,
    ) -> Option<Vec<Link<LinkProperties>>> {
        if let Some(link_options) = bucket.link_options_for(
            self.device_info.id,
            &self.device_info.agent_type,
            target_class,
        ) {
            let mut stats: Vec<&CommStats> = Vec::with_capacity(link_options.len());
            link_options.iter().for_each(|link| {
                if let Some(link_stats) = bucket.stats_for(&link.target) {
                    stats.push(link_stats);
                }
            });
            return self.select_links(link_options, target_class, &stats);
        }
        None
    }

    fn send_payload(
        &mut self,
        links: Vec<Link<LinkProperties>>,
        target_class: &AgentClass,
        payload: FlPayload,
        rx_payloads: &Option<Vec<FlPayload>>,
        bucket: &mut FlBucket<B>,
        actions: HashMap<MessageType, Action>,
    ) {
        links.into_iter().for_each(|target_link| {
            let mut prepared_payload =
                set_actions_before_tx(payload.clone(), target_link.target, &actions);

            // If we know about the target agent, take payload forwarding decisions.
            if let Some(target_state) = bucket.agent_data_of(&target_link.target) {
                if let Some(ref payloads) = rx_payloads {
                    let mut units = filter_units_to_fwd(target_state, payloads);
                    self.models
                        .composer
                        .update_metadata(&mut prepared_payload, &units);
                    prepared_payload.data_units.append(&mut units);
                }
            }

            if let Some(writer) = &mut bucket.models.results.payload_tx {
                writer.add_data(self.build_update(target_link.target, &prepared_payload));
            }

            if target_class == &self.device_info.agent_class {
                self.transmit_sl(prepared_payload, target_link, bucket);
            } else {
                self.transmit(prepared_payload, target_link, bucket);
            }
        });
    }

    fn send_fl_message(
        &mut self,
        bucket: &mut FlBucket<B>,
        target_classes: &[AgentClass],
        rx_payloads: &Option<Vec<FlPayload>>,
    ) {
        target_classes.iter().for_each(|target_class| {
            if let Some(links) = self.get_links(target_class, bucket) {
                let payload =
                    self.models
                        .composer
                        .compose(self.step, target_class, &self.device_info);
                let mut actions = self.models.actor.actions_for(target_class).to_owned();

                if let Some(draft) = &self.models.composer.peek_draft() {
                    if let Some(agents) = &draft.selected_clients {
                        actions
                            .values_mut()
                            .for_each(|action| action.update_broadcast(agents));
                    }
                }
                self.send_payload(links, target_class, payload, rx_payloads, bucket, actions);
            }
        });
    }

    fn build_update(&self, target_id: AgentId, payload: &FlPayload) -> PayloadUpdate {
        let mut payload_types = String::new();
        payload_types.push('|');
        payload.data_units.iter().for_each(|unit| {
            payload_types.push_str(unit.message_type.to_string().as_str());
            payload_types.push('|');
        });

        let mut action_types_str = String::new();
        action_types_str.push('|');
        payload.data_units.iter().for_each(|unit| {
            action_types_str.push_str(unit.action.action_type.to_string().as_str());
            action_types_str.push('|');
        });

        let mut fl_content_type_str = String::new();
        fl_content_type_str.push('|');
        payload.data_units.iter().for_each(|unit| {
            fl_content_type_str.push_str(unit.fl_action.to_string().as_str());
            fl_content_type_str.push('|');
        });

        PayloadUpdate::builder()
            .time_step(self.step)
            .source(self.device_info.id)
            .target(target_id)
            .agent_state(self.fl_agent.agent_state())
            .payload_type(payload_types)
            .action_type(action_types_str)
            .fl_content(payload.data_units.first().unwrap().fl_action.to_string())
            .quantity(payload.metadata.total_count)
            .payload_size(payload.metadata.total_size.as_u64())
            .build()
    }
}

impl<B: AutodiffBackend> Activatable<FlBucket<B>> for Device<B> {
    fn activate(&mut self, bucket: &mut FlBucket<B>) {
        trace!("Starting agent: {}", self.device_info.id);
        self.power_state = PowerState::On;
        bucket.update_agent_data_of(self.device_info.id, self.device_info);
        self.fl_agent.init(bucket);
    }

    fn deactivate(&mut self) {
        self.power_state = PowerState::Off;
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

impl<B: AutodiffBackend> Orderable for Device<B> {
    fn order(&self) -> AgentOrder {
        self.device_info.agent_order
    }
}

impl<B: AutodiffBackend> Movable<FlBucket<B>> for Device<B> {
    type M = MapState;

    fn mobility(&self) -> &Self::M {
        &self.map_state
    }

    fn set_mobility(&mut self, bucket: &mut FlBucket<B>) {
        self.map_state = bucket
            .positions_for(self.device_info.id, &self.device_info.agent_type)
            .unwrap_or(self.map_state);

        if let Some(writer) = &mut bucket.models.results.positions {
            writer.add_data(self.step, self.device_info.id, &self.map_state)
        }
    }
}

impl<B: AutodiffBackend>
    Transmitter<
        FlBucket<B>,
        MessageType,
        MessageUnit,
        LinkProperties,
        FlPayloadInfo,
        DeviceInfo,
        Message,
    > for Device<B>
{
    fn transmit(
        &mut self,
        payload: FlPayload,
        target_link: Link<LinkProperties>,
        bucket: &mut FlBucket<B>,
    ) {
        self.models.flow.register_outgoing_attempt(&payload);
        let tx_metrics = bucket.models.network.transfer(&payload);

        let tx_stats = TxData::builder()
            .agent_id(self.device_info.id.as_u64())
            .selected_agent(target_link.target.as_u64())
            .distance(target_link.properties.distance.unwrap_or(-1.0))
            .data_count(payload.metadata.total_count)
            .link_found(self.step.as_u64())
            .tx_order(tx_metrics.tx_order)
            .tx_status(0)
            .payload_size(tx_metrics.payload_size.as_u64())
            .tx_fail_reason(0)
            .latency(0)
            .build();

        if let Some(tx) = &mut bucket.models.results.tx_data {
            tx.add_data(self.step, tx_stats);
        }

        self.models.flow.register_outgoing_feasible(&payload);
        bucket
            .models
            .data_lake
            .add_payload_to(target_link.target, payload);
    }

    fn transmit_sl(
        &mut self,
        payload: FlPayload,
        target_link: Link<LinkProperties>,
        bucket: &mut FlBucket<B>,
    ) {
        self.models.sl_flow.register_outgoing_attempt(&payload);
        let sl_metrics = bucket.models.network.transfer(&payload);

        let tx_stats = TxData::builder()
            .agent_id(self.device_info.id.as_u64())
            .selected_agent(target_link.target.as_u64())
            .distance(target_link.properties.distance.unwrap_or(-1.0))
            .data_count(payload.metadata.total_count)
            .link_found(self.step.as_u64())
            .tx_order(sl_metrics.tx_order)
            .tx_status(0)
            .payload_size(sl_metrics.payload_size.as_u64())
            .tx_fail_reason(0)
            .latency(0)
            .build();

        if let Some(tx) = &mut bucket.models.results.tx_data {
            tx.add_data(self.step, tx_stats);
        }

        self.models.sl_flow.register_outgoing_feasible(&payload);
        bucket
            .models
            .data_lake
            .add_sl_payload_to(target_link.target, payload);
    }
}

impl<B: AutodiffBackend>
    Receiver<FlBucket<B>, MessageType, MessageUnit, FlPayloadInfo, DeviceInfo, Message>
    for Device<B>
{
    fn receive(&mut self, bucket: &mut FlBucket<B>) -> Option<Vec<FlPayload>> {
        bucket.models.data_lake.payloads_for(self.device_info.id)
    }

    fn receive_sl(&mut self, bucket: &mut FlBucket<B>) -> Option<Vec<FlPayload>> {
        bucket.models.data_lake.sl_payloads_for(self.device_info.id)
    }
}

impl<B: AutodiffBackend> Agent<FlBucket<B>> for Device<B> {
    fn id(&self) -> AgentId {
        self.device_info.id
    }

    fn stage_one(&mut self, bucket: &mut FlBucket<B>) {
        self.step = bucket.step;
        self.fl_agent.update_step(self.step);
        self.set_mobility(bucket);

        bucket.update_stats_of(self.device_info.id, self.stats);
        debug!(
            "Uplink stage for agent: {} id at step: {}",
            self.device_info.id, self.step
        );
        self.models.flow.reset();

        let mut rx_payloads = self.receive(bucket);
        if let Some(ref mut payloads) = rx_payloads {
            self.models.flow.register_incoming(payloads);
            self.fl_agent.handle_incoming(bucket, payloads);
            payloads.iter_mut().for_each(|payload| {
                complete_actions(payload, &self.device_info);
            });
        }

        let mut target_classes = self.models.directions.stage_one.target_classes.to_vec();
        if !target_classes.is_empty() {
            let message_draft = self.fl_agent.draft_fl_message(bucket);
            self.models.composer.update_draft(message_draft);
            self.send_fl_message(bucket, &target_classes, &rx_payloads);
        }

        if self.models.directions.stage_one.is_sidelink {
            target_classes.clear();
            target_classes.push(self.device_info.agent_class);
            self.send_fl_message(bucket, &target_classes, &rx_payloads);
        }
        self.models.composer.reset_draft();
    }

    fn stage_two_reverse(&mut self, bucket: &mut FlBucket<B>) {
        debug!(
            "Downlink stage for agent: {} id at step: {}",
            self.device_info.id, self.step
        );

        let mut rx_payloads = self.receive(bucket);
        if let Some(ref mut payloads) = rx_payloads {
            self.models.flow.register_incoming(payloads);
            self.fl_agent.handle_incoming(bucket, payloads);
            payloads.iter_mut().for_each(|payload| {
                complete_actions(payload, &self.device_info);
            });
        }

        let mut target_classes = self.models.directions.stage_two.target_classes.to_vec();
        if !target_classes.is_empty() {
            let message_draft = self.fl_agent.draft_fl_message(bucket);
            debug!("draft quantity is {}", message_draft.quantity);
            self.models.composer.update_draft(message_draft);
            self.send_fl_message(bucket, &target_classes, &rx_payloads);
        }

        if self.models.directions.stage_two.is_sidelink {
            target_classes.clear();
            target_classes.push(self.device_info.agent_class);
            self.send_fl_message(bucket, &target_classes, &rx_payloads);
        }
        self.models.composer.reset_draft();
    }

    fn stage_three(&mut self, bucket: &mut FlBucket<B>) {
        self.receive_sl(bucket);
    }

    fn stage_four_reverse(&mut self, bucket: &mut FlBucket<B>) {
        if let Some(rx) = &mut bucket.models.results.rx_counts {
            rx.add_data(
                self.step,
                self.device_info.id,
                &self.models.flow.comm_stats.outgoing_stats,
            );
        }

        if self.step == self.models.power.peek_time_to_off() {
            self.power_state = PowerState::Off;
        }
    }

    fn stage_five(&mut self, _bucket: &mut FlBucket<B>) {
        self.stats = self.models.flow.comm_stats;
    }
}
