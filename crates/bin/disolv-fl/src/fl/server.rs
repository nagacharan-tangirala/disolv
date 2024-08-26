use burn::tensor::backend::{AutodiffBackend, Backend};
use log::{debug, error};
use log::__private_api::loc;
use typed_builder::TypedBuilder;

use disolv_core::agent::{
    Activatable, Agent, AgentClass, AgentId, AgentKind, AgentOrder, AgentProperties, Movable,
    Orderable,
};
use disolv_core::bucket::{Bucket, TimeMS};
use disolv_core::hashbrown::HashMap;
use disolv_core::message::Payload;
use disolv_core::radio::{Action, ActionType, Link, Receiver, Transmitter};
use disolv_models::device::actions::{
    am_i_target, complete_actions, filter_units_to_fwd, set_actions_before_tx,
};
use disolv_models::device::actor::Actor;
use disolv_models::device::directions::Directions;
use disolv_models::device::flow::FlowRegister;
use disolv_models::device::mobility::MapState;
use disolv_models::device::models::{Compose, LinkSelect};
use disolv_models::device::power::{PowerManager, PowerState};
use disolv_models::net::radio::{CommStats, LinkProperties};

use crate::fl::bucket::FlBucket;
use crate::fl::client::AgentInfo;
use crate::models::ai::aggregate::Aggregator;
use crate::models::ai::compose::{FlComposer, FlMessageToBuild};
use crate::models::ai::distribute::DataDistributor;
use crate::models::ai::mnist::MnistModel;
use crate::models::ai::models::{FlAgent, ModelType, TrainerType};
use crate::models::ai::times::ServerTimes;
use crate::models::device::compose::V2XComposer;
use crate::models::device::energy::EnergyType;
use crate::models::device::link::LinkSelector;
use crate::models::device::message::{FlPayload, FlPayloadInfo, Message, MessageType, MessageUnit};
use crate::models::device::select::ClientSelector;

#[derive(Default, Copy, Clone, Debug)]
pub(crate) enum ServerState {
    #[default]
    Idle,
    ClientAnalysis,
    ClientSelection,
    TrainingRound,
    Aggregation,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct ServerModels {
    pub power: PowerManager,
    pub flow: FlowRegister,
    pub sl_flow: FlowRegister,
    pub composer: V2XComposer,
    pub actor: Actor<MessageType>,
    pub directions: Directions,
    pub energy: EnergyType,
    pub link_selector: Vec<(AgentClass, LinkSelector)>,
}

#[derive(Clone, TypedBuilder)]
pub(crate) struct FlModels<B: AutodiffBackend> {
    pub(crate) client_classes: Vec<AgentClass>,
    pub(crate) trainer: Trainer<B>,
    pub(crate) composer: FlComposer,
    pub(crate) client_selector: ClientSelector,
    pub(crate) times: ServerTimes,
    pub(crate) aggregator: Aggregator<B>,
}

#[derive(Clone, TypedBuilder)]
pub(crate) struct Server<B: AutodiffBackend> {
    pub(crate) server_info: AgentInfo,
    pub(crate) server_state: ServerState,
    pub(crate) models: ServerModels,
    pub(crate) fl_models: FlModels<B>,
    #[builder(default)]
    pub(crate) step: TimeMS,
    #[builder(default)]
    pub(crate) power_state: PowerState,
    #[builder(default)]
    pub(crate) map_state: MapState,
    #[builder(default)]
    pub(crate) stats: CommStats,
}

impl<B: AutodiffBackend> Server<B> {
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
        &mut self,
        target_class: &AgentClass,
        bucket: &mut FlBucket<B>,
    ) -> Option<Vec<Link<LinkProperties>>> {
        let link_options = match bucket.link_options_for(
            self.server_info.id,
            &self.server_info.agent_type,
            target_class,
        ) {
            Some(links) => links,
            None => return None,
        };

        let mut stats: Vec<&CommStats> = Vec::with_capacity(link_options.len());
        link_options.iter().for_each(|link| {
            let link_stats = bucket.stats_for(&link.target);
            if link_stats.is_some() {
                stats.push(link_stats.unwrap());
            }
        });
        self.select_links(link_options, target_class, &stats)
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
            let mut this_payload = payload.clone();

            // If we know about the target agent, take payload forwarding decisions.
            if let Some(target_state) = bucket.agent_data_of(&target_link.target) {
                match rx_payloads {
                    Some(ref payloads) => {
                        let mut units = filter_units_to_fwd(target_state, payloads);
                        self.models
                            .composer
                            .append_units_to(&mut this_payload, &mut units);
                    }
                    None => (),
                }
            }

            let prepared_payload = set_actions_before_tx(this_payload, &actions);
            if target_class == &self.server_info.agent_class {
                self.transmit_sl(prepared_payload, target_link, bucket);
            } else {
                self.transmit(prepared_payload, target_link, bucket);
            }
        });
    }

    fn send_fl_message(
        &mut self,
        bucket: &mut FlBucket<B>,
        message_to_build: FlMessageToBuild,
        broadcast: Option<Vec<AgentId>>,
    ) {
        self.models
            .directions
            .stage_two
            .target_classes
            .clone()
            .iter_mut()
            .for_each(|target_class| match self.get_links(target_class, bucket) {
                Some(links) => {
                    self.fl_models
                        .composer
                        .set_message_to_build(message_to_build);
                    let payload = self.fl_models.composer.compose_payload(self.server_info);
                    let mut actions = self.models.actor.actions_for(target_class).to_owned();

                    if let Some(agents) = broadcast.clone() {
                        actions.clone().values_mut().for_each(|action| {
                            match &mut action.to_broadcast {
                                Some(ref mut broadcast_vec) => broadcast_vec.extend(agents.iter()),
                                None => {}
                            }
                        });
                    }
                    self.send_payload(links, target_class, payload, &None, bucket, actions);
                }
                None => {}
            });
    }

    fn do_fl_actions(&mut self, bucket: &mut FlBucket<B>, payload: &mut FlPayload) {
        match payload.query_type {
            Message::StateInfo => self.register_client(bucket, payload),
            Message::LocalModel => self.collect_local_model(bucket, payload),
            _ => panic!("Server should not receive this message"),
        }

        // Mark the payload to be consumed.
        payload
            .data_units
            .iter_mut()
            .for_each(|message| message.action.action_type = ActionType::Consume);
    }

    fn register_client(&mut self, bucket: &mut FlBucket<B>, payload: &mut FlPayload) {
        match bucket.agent_data_of(&payload.agent_state.id) {
            Some(client_info) => self.fl_models.client_selector.register_client(client_info),
            None => panic!("bucket does not know about this client"),
        }
    }

    fn collect_local_model(&mut self, bucket: &mut FlBucket<B>, payload: &mut FlPayload) {
        let local_model = bucket
            .models
            .model_lake
            .local_model_of(payload.agent_state.id);
        self.fl_models.aggregator.add_local_model(local_model);
    }

    fn handle_initiation(&mut self, bucket: &mut FlBucket<B>) {
        debug!("Changing from idle to analysis at {}", self.step);
        self.fl_models
            .times
            .update_time(self.step, self.server_state);
        self.server_state = ServerState::ClientAnalysis;

        let message_to_build = FlMessageToBuild::builder()
            .message(Message::StateInfo)
            .message_type(MessageType::KiloByte)
            .quantity(1)
            .build();
        let selected_clients = self.fl_models.client_selector.selected_clients().to_owned();
        self.send_fl_message(bucket, message_to_build, Some(selected_clients));
    }

    fn handle_analysis(&mut self, bucket: &mut FlBucket<B>) {
        debug!("Changing from analysis to selection at {}", self.step);
        self.fl_models
            .times
            .update_time(self.step, self.server_state);
        self.server_state = ServerState::ClientSelection;

        let message_to_build = FlMessageToBuild::builder()
            .message(Message::GlobalModel)
            .message_type(MessageType::F64Weights)
            .quantity(self.fl_models.trainer.no_of_weights())
            .build();
        let selected_clients = self.fl_models.client_selector.selected_clients().to_owned();
        self.send_fl_message(bucket, message_to_build, Some(selected_clients));
    }

    fn handle_selection(&mut self, bucket: &mut FlBucket<A, B>) {
        debug!("Changing from selection to training at {}", self.step);
        self.server_state = ServerState::TrainingRound;

        let message_to_build = FlMessageToBuild::builder()
            .message(Message::InitiateTraining)
            .message_type(MessageType::KiloByte)
            .quantity(1)
            .build();
        let selected_clients = self.fl_models.client_selector.selected_clients().to_owned();
        self.send_fl_message(bucket, message_to_build, Some(selected_clients));
    }

    fn handle_training(&mut self, bucket: &mut FlBucket<B>) {
        debug!("Changing from training to aggregation at {}", self.step);
        self.server_state = ServerState::Aggregation;

        let message_to_build = FlMessageToBuild::builder()
            .message(Message::CompleteTraining)
            .message_type(MessageType::KiloByte)
            .quantity(1)
            .build();
        let selected_clients = self.fl_models.client_selector.selected_clients().to_owned();
        self.send_fl_message(bucket, message_to_build, Some(selected_clients));
    }

    fn handle_aggregation(&mut self, bucket: &mut FlBucket<A, B>) {
        debug!("Changing from aggregation to initiation at {}", self.step);
        self.server_state = ServerState::Idle;

        self.fl_models.global_model = self.fl_models.aggregator.aggregate(
            self.fl_models.global_model.clone(),
            self.fl_models.trainer.device(),
        );

        let message_to_build = FlMessageToBuild::builder()
            .message(Message::CompleteTraining)
            .message_type(MessageType::KiloByte)
            .quantity(1)
            .build();
        let selected_clients = self.fl_models.client_selector.selected_clients().to_owned();
        self.send_fl_message(bucket, message_to_build, Some(selected_clients));
    }
}

impl<B: AutodiffBackend> Activatable<FlBucket<B>> for Server<B> {
    fn activate(&mut self, bucket: &mut FlBucket<B>) {
        debug!("Starting agent: {}", self.server_info.id);
        self.power_state = PowerState::On;
        bucket.update_agent_data_of(self.server_info.id, self.server_info);
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

impl<B: AutodiffBackend> Orderable for Server<B> {
    fn order(&self) -> AgentOrder {
        self.server_info.agent_order
    }
}

impl<B: AutodiffBackend> Movable<FlBucket<B>> for Server<B> {
    type M = MapState;

    fn mobility(&self) -> &Self::M {
        &self.map_state
    }

    fn set_mobility(&mut self, bucket: &mut FlBucket<B>) {
        self.map_state = bucket
            .positions_for(self.server_info.id, &self.server_info.agent_type)
            .unwrap_or(self.map_state);
        bucket.models.output.basic_results.positions.add_data(
            self.step,
            self.server_info.id,
            &self.map_state,
        );
    }
}

impl<B: AutodiffBackend>
    Transmitter<
        FlBucket<B>,
        MessageType,
        MessageUnit,
        LinkProperties,
        FlPayloadInfo,
        AgentInfo,
        Message,
    > for Server<B>
{
    fn transmit(
        &mut self,
        payload: FlPayload,
        target: Link<LinkProperties>,
        bucket: &mut FlBucket<B>,
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
        bucket: &mut FlBucket<B>,
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

impl<B: AutodiffBackend>
    Receiver<FlBucket<B>, MessageType, MessageUnit, FlPayloadInfo, AgentInfo, Message>
    for Server<B>
{
    fn receive(&mut self, bucket: &mut FlBucket<B>) -> Option<Vec<FlPayload>> {
        bucket.models.data_lake.payloads_for(self.server_info.id)
    }

    fn receive_sl(&mut self, bucket: &mut FlBucket<B>) -> Option<Vec<FlPayload>> {
        bucket.models.data_lake.sl_payloads_for(self.server_info.id)
    }
}

impl<B: AutodiffBackend> Agent<FlBucket<B>> for Server<B> {
    fn id(&self) -> AgentId {
        self.server_info.id
    }

    fn stage_one(&mut self, bucket: &mut FlBucket<B>) {
        self.step = bucket.step;
        self.set_mobility(bucket);
        bucket.update_stats_of(self.server_info.id, self.stats);
        debug!(
            "Uplink stage for agent: {} id at step: {}",
            self.server_info.id, self.step
        );
        self.models.flow.reset();

        let mut rx_payloads = self.receive(bucket);
        self.handle_fl_messages(bucket, &mut rx_payloads);

        if let Some(ref mut payloads) = rx_payloads {
            self.models.flow.register_incoming(payloads);
            payloads.iter_mut().for_each(|payload| {
                complete_actions(payload, &self.server_info);
            });
        }

        self.models
            .directions
            .stage_two
            .target_classes
            .clone()
            .iter()
            .for_each(|target_class| match self.get_links(target_class, bucket) {
                Some(links) => {
                    let payload =
                        self.models
                            .composer
                            .compose(self.step, target_class, &self.server_info);
                    let actions = self.models.actor.actions_for(target_class).to_owned();
                    self.send_payload(links, target_class, payload, &rx_payloads, bucket, actions);
                }
                None => {}
            });

        // Talk to the agents in side link.
        if self.models.directions.stage_one.is_sidelink {
            let target_class = self.server_info.agent_class.clone();
            match self.get_links(&target_class, bucket) {
                Some(links) => {
                    let payload =
                        self.models
                            .composer
                            .compose(self.step, &target_class, &self.server_info);
                    let actions = self.models.actor.actions_for(&target_class).to_owned();
                    self.send_payload(links, &target_class, payload, &rx_payloads, bucket, actions);
                }
                None => {}
            }
        }
    }

    fn stage_two_reverse(&mut self, bucket: &mut FlBucket<B>) {
        // Depending on the training state, send messages to clients.
        if self.fl_models.times.is_time_to_change(self.step) {
            match self.server_state {
                ServerState::Idle => self.handle_initiation(bucket),
                ServerState::ClientAnalysis => self.handle_analysis(bucket),
                ServerState::ClientSelection => self.handle_selection(bucket),
                ServerState::TrainingRound => self.handle_training(bucket),
                ServerState::Aggregation => self.handle_aggregation(bucket),
            }
        }
    }

    fn stage_three(&mut self, bucket: &mut FlBucket<B>) {
        // Receive data from the peers.
        self.receive_sl(bucket);
    }

    fn stage_four_reverse(&mut self, bucket: &mut FlBucket<B>) {
        debug!(
            "Downlink stage for agent: {} id at step: {}",
            self.server_info.id, self.step
        );

        bucket.models.output.basic_results.rx_counts.add_data(
            self.step,
            self.server_info.id,
            &self.models.flow.comm_stats.outgoing_stats,
        );

        if self.step == self.models.power.peek_time_to_off() {
            self.power_state = PowerState::Off;
        }
    }

    fn stage_five(&mut self, _bucket: &mut FlBucket<B>) {
        self.stats = self.models.flow.comm_stats;
    }
}

impl<B: AutodiffBackend>
    FlAgent<FlBucket<B>, MessageType, MessageUnit, FlPayloadInfo, AgentInfo, Message>
    for Server<B>
{
    fn handle_fl_messages(
        &mut self,
        bucket: &mut FlBucket<B>,
        messages: &mut Option<Vec<FlPayload>>,
    ) {
        if let Some(payloads) = messages {
            payloads.iter_mut().for_each(|payload| {
                match payload.query_type {
                    Message::Sensor => {}
                    _ => self.do_fl_actions(bucket, payload),
                };
            });
        }
    }

    fn update_state(&mut self) {
        todo!()
    }
}
