use burn::prelude::Backend;
use burn::tensor::backend::AutodiffBackend;
use log::debug;
use typed_builder::TypedBuilder;

use disolv_core::agent::{
    Activatable, Agent, AgentClass, AgentId, AgentKind, AgentOrder, AgentProperties, Movable,
    Orderable,
};
use disolv_core::bucket::TimeMS;
use disolv_core::metrics::Bytes;
use disolv_core::radio::{Action, ActionType, Link, Receiver, Transmitter};
use disolv_models::device::actions::{
    complete_actions, filter_units_to_fwd, set_actions_before_tx,
};
use disolv_models::device::actor::Actor;
use disolv_models::device::flow::FlowRegister;
use disolv_models::device::metrics::MegaHertz;
use disolv_models::device::mobility::MapState;
use disolv_models::device::models::{Compose, LinkSelect};
use disolv_models::device::power::{PowerManager, PowerState};
use disolv_models::net::metrics::Bandwidth;
use disolv_models::net::radio::{CommStats, LinkProperties};

use crate::fl::bucket::FlBucket;
use crate::models::ai::compose::{FlComposer, FlMessageToBuild};
use crate::models::ai::models::{ClientState, DatasetType, ModelType};
use crate::models::device::compose::{V2XComposer, V2XDataSource};
use crate::models::device::energy::EnergyType;
use crate::models::device::hardware::Hardware;
use crate::models::device::link::LinkSelector;
use crate::models::device::message::{FlPayload, FlPayloadInfo, Message, MessageType, MessageUnit};

#[derive(Clone, Copy, Debug, Default, TypedBuilder)]
pub(crate) struct AgentInfo {
    pub(crate) id: AgentId,
    pub(crate) agent_type: AgentKind,
    pub(crate) agent_class: AgentClass,
    pub(crate) agent_order: AgentOrder,
    pub(crate) cpu: MegaHertz,
    pub(crate) memory: Bytes,
    pub(crate) bandwidth: Bandwidth,
}

impl AgentProperties for AgentInfo {
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
pub(crate) struct ClientModels {
    pub(crate) power: PowerManager,
    pub(crate) flow: FlowRegister,
    pub(crate) sl_flow: FlowRegister,
    pub(crate) v2x_composer: V2XComposer,
    pub(crate) actor: Actor<MessageType>,
    pub(crate) energy: EnergyType,
    pub(crate) hardware: Hardware,
    pub(crate) selector: Vec<(AgentClass, LinkSelector)>,
}

#[derive(Clone, TypedBuilder)]
pub(crate) struct FlModels<A: AutodiffBackend, B: Backend> {
    pub(crate) dataset_type: DatasetType,
    pub(crate) model_type: ModelType<A, B>,
    pub(crate) composer: FlComposer,
}

#[derive(Clone, TypedBuilder)]
pub(crate) struct Client<A: AutodiffBackend, B: Backend> {
    pub(crate) client_info: AgentInfo,
    pub(crate) training_state: ClientState,
    pub(crate) models: ClientModels,
    pub(crate) fl_models: FlModels<A, B>,
    #[builder(default)]
    pub(crate) step: TimeMS,
    #[builder(default)]
    pub(crate) power_state: PowerState,
    #[builder(default)]
    pub(crate) map_state: MapState,
    #[builder(default)]
    pub(crate) stats: CommStats,
}

impl<A: AutodiffBackend, B: Backend> Client<A, B> {
    fn select_links(
        &self,
        link_options: Vec<Link<LinkProperties>>,
        target_class: &AgentClass,
        stats: &Vec<&CommStats>,
    ) -> Option<Vec<Link<LinkProperties>>> {
        for selectors in self.models.selector.iter() {
            if selectors.0 == *target_class {
                return Some(selectors.1.select_link(link_options, stats));
            }
        }
        None
    }

    fn talk_to_class(
        &mut self,
        target_class: AgentClass,
        rx_payloads: &Option<Vec<FlPayload>>,
        bucket: &mut FlBucket,
    ) {
        let link_options = match bucket.link_options_for(
            self.client_info.id,
            &self.client_info.agent_type,
            &target_class,
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

        let targets = match self.select_links(link_options, &target_class, &stats) {
            Some(links) => links,
            None => return,
        };

        let payload = self
            .models
            .v2x_composer
            .compose(self.step, &target_class, &self.client_info);

        if target_class == AgentClass::FlServer {
            let fl_message = self.get_fl_message();
        }

        targets.into_iter().for_each(|target_link| {
            let mut this_payload = payload.clone();
            if let Some(target_state) = bucket.agent_data_of(&target_link.target) {
                match rx_payloads {
                    Some(ref payloads) => {
                        let mut units = filter_units_to_fwd(target_state, payloads);
                        self.models
                            .v2x_composer
                            .append_units_to(&mut this_payload, &mut units);
                    }
                    None => (),
                }
            }

            let actions = self.models.actor.actions_for(&target_class);
            let prepared_payload = set_actions_before_tx(this_payload, actions);
            if target_class == self.client_info.agent_class {
                self.transmit_sl(prepared_payload, target_link, bucket);
            } else {
                self.transmit(prepared_payload, target_link, bucket);
            }
        });
    }

    fn get_fl_message(&mut self) -> Option<MessageUnit> {
        let message_type = match self.training_state {
            ClientState::Ongoing => return None,
            ClientState::Fail => return None,
            ClientState::Sensing => MessageType::KiloByte,
            ClientState::Selected => MessageType::KiloByte,
            ClientState::WaitingForGlobalModel => MessageType::F64Weights,
        };
        let message = match self.training_state {
            ClientState::Ongoing => return None,
            ClientState::Fail => return None,
            ClientState::Sensing => MessageType::KiloByte,
            ClientState::Selected => MessageType::KiloByte,
            ClientState::WaitingForGlobalModel => MessageType::F64Weights,
        };
        let message_to_build = FlMessageToBuild::builder()
            .message(message)
            .message_type(message_type)
            .quantity()
            .build();

        Some(self.fl_models.composer.compose_data_unit(message_to_build))
    }
}

impl<A: AutodiffBackend, B: Backend> Activatable<FlBucket> for Client<A, B> {
    fn activate(&mut self, bucket: &mut FlBucket) {
        self.power_state = PowerState::On;
        bucket.update_agent_data_of(self.client_info.id, self.client_info);
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

impl<A: AutodiffBackend, B: Backend> Orderable for Client<A, B> {
    fn order(&self) -> AgentOrder {
        self.client_info.agent_order
    }
}

impl<A: AutodiffBackend, B: Backend> Movable<FlBucket> for Client<A, B> {
    type M = MapState;

    fn mobility(&self) -> &Self::M {
        &self.map_state
    }

    fn set_mobility(&mut self, bucket: &mut FlBucket) {
        self.map_state = bucket
            .positions_for(self.client_info.id, &self.client_info.agent_type)
            .unwrap_or(self.map_state);
        bucket.models.output.basic_results.positions.add_data(
            self.step,
            self.client_info.id,
            &self.map_state,
        );
    }
}

impl<A: AutodiffBackend, B: Backend>
    Transmitter<
        FlBucket,
        MessageType,
        MessageUnit,
        LinkProperties,
        FlPayloadInfo,
        AgentInfo,
        Message,
    > for Client<A, B>
{
    fn transmit(
        &mut self,
        payload: FlPayload,
        target: Link<LinkProperties>,
        bucket: &mut FlBucket,
    ) {
        debug!(
            "Transmitting payload from agent {} to agent {} with blobs {}",
            payload.agent_state.id(),
            target.target,
            payload.data_units.len()
        );

        self.models.flow.register_outgoing_attempt(&payload);
        let tx_metrics = bucket.models.network.transfer(&payload);
        match &mut bucket.models.output.tx_data_writer {
            Some(tx) => tx.add_data(self.step, &target, &payload, tx_metrics),
            None => {}
        }

        self.models.flow.register_outgoing_feasible(&payload);
        bucket
            .models
            .data_lake
            .add_payload_to(target.target, payload);
    }

    fn transmit_sl(
        &mut self,
        payload: FlPayload,
        target: Link<LinkProperties>,
        bucket: &mut FlBucket,
    ) {
        debug!(
            "Transmitting SL payload from agent {} to agent {}",
            payload.agent_state.id(),
            target.target
        );

        self.models.sl_flow.register_outgoing_attempt(&payload);
        let sl_metrics = bucket.models.network.transfer(&payload);

        match &mut bucket.models.output.tx_data_writer {
            Some(tx) => tx.add_data(self.step, &target, &payload, sl_metrics),
            None => {}
        }

        self.models.sl_flow.register_outgoing_feasible(&payload);
        bucket
            .models
            .data_lake
            .add_sl_payload_to(target.target, payload);
    }
}

impl<A: AutodiffBackend, B: Backend>
    Receiver<FlBucket, MessageType, MessageUnit, FlPayloadInfo, AgentInfo, Message>
    for Client<A, B>
{
    fn receive(&mut self, bucket: &mut FlBucket) -> Option<Vec<FlPayload>> {
        bucket.models.data_lake.payloads_for(self.client_info.id)
    }

    fn receive_sl(&mut self, bucket: &mut FlBucket) -> Option<Vec<FlPayload>> {
        bucket.models.data_lake.sl_payloads_for(self.client_info.id)
    }
}

impl<A: AutodiffBackend, B: Backend> Agent<FlBucket> for Client<A, B> {
    type P = AgentInfo;

    fn id(&self) -> AgentId {
        self.client_info.id
    }

    fn stage_one(&mut self, bucket: &mut FlBucket) {
        self.step = bucket.step;
        self.set_mobility(bucket);
        bucket.update_agent_data_of(self.client_info.id, self.client_info);
        bucket.update_stats_of(self.client_info.id, self.stats);
        debug!(
            "Uplink stage for agent: {} id at step: {}",
            self.client_info.id, self.step
        );
        self.models.flow.reset();

        let mut rx_payloads = self.receive(bucket);
        if let Some(ref mut payloads) = rx_payloads {
            self.models.flow.register_incoming(payloads);
            payloads.iter_mut().for_each(|payload| {
                complete_actions(payload, &self.client_info);
            });
        }

        // Take FL payloads and use the data.

        for target_class in self.models.actor.target_classes.clone().into_iter() {
            self.talk_to_class(target_class, &rx_payloads, bucket);
        }
        self.talk_to_class(self.client_info.agent_class.clone(), &rx_payloads, bucket);
    }

    fn stage_two_reverse(&mut self, _bucket: &mut FlBucket) {}

    fn stage_three(&mut self, bucket: &mut FlBucket) {
        // Receive data from the peers.
        self.receive_sl(bucket);
    }

    fn stage_four_reverse(&mut self, bucket: &mut FlBucket) {
        debug!(
            "Downlink stage for agent: {} id at step: {}",
            self.client_info.id, self.step
        );

        bucket.models.output.basic_results.rx_counts.add_data(
            self.step,
            self.client_info.id,
            &self.models.flow.comm_stats.outgoing_stats,
        );

        if self.step == self.models.power.peek_time_to_off() {
            self.power_state = PowerState::Off;
        }
    }
}
