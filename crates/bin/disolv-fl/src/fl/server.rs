use arrow::compute::take_arrays;
use burn::tensor::backend::AutodiffBackend;
use disolv_core::agent::{
    Activatable, Agent, AgentClass, AgentId, AgentKind, AgentOrder, AgentProperties, Movable,
    Orderable,
};
use disolv_core::bucket::TimeMS;
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
use log::debug;
use typed_builder::TypedBuilder;

use crate::fl::bucket::FlBucket;
use crate::fl::client::AgentInfo;
use crate::models::ai::aggregate::Aggregator;
use crate::models::ai::compose::{FlComposer, FlMessageToBuild};
use crate::models::ai::data::DataHolder;
use crate::models::ai::mnist::MnistModel;
use crate::models::ai::models::ModelType;
use crate::models::ai::select::ClientSelector;
use crate::models::ai::times::ServerTimes;
use crate::models::ai::trainer::Trainer;
use crate::models::device::energy::EnergyType;
use crate::models::device::link::LinkSelector;
use crate::models::device::message::{
    FlContent, FlPayload, FlPayloadInfo, Message, MessageType, MessageUnit,
};

#[derive(Default, Copy, Clone, Debug)]
pub(crate) enum ServerState {
    #[default]
    Idle,
    ClientAnalysis,
    ClientSelection,
    TrainingRound,
    Aggregation,
}

impl ServerState {
    pub const fn value(&self) -> u64 {
        match self {
            ServerState::Idle => 1000,
            ServerState::ClientAnalysis => 2000,
            ServerState::ClientSelection => 3000,
            ServerState::TrainingRound => 4000,
            ServerState::Aggregation => 5000,
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct ServerModels {
    pub power: PowerManager,
    pub flow: FlowRegister,
    pub sl_flow: FlowRegister,
    pub actor: Actor<MessageType>,
    pub directions: Directions,
    pub energy: EnergyType,
    pub link_selector: Vec<(AgentClass, LinkSelector)>,
}

#[derive(Clone, TypedBuilder)]
pub(crate) struct FlServerModels<B: AutodiffBackend> {
    pub(crate) client_classes: Vec<AgentClass>,
    pub(crate) trainer: Trainer<B>,
    pub(crate) composer: FlComposer,
    pub(crate) client_selector: ClientSelector,
    pub(crate) times: ServerTimes,
    pub(crate) aggregator: Aggregator<B>,
    pub(crate) holder: DataHolder,
}

#[derive(Clone, TypedBuilder)]
pub(crate) struct Server<B: AutodiffBackend> {
    pub(crate) server_info: AgentInfo,
    pub(crate) models: ServerModels,
    pub(crate) fl_models: FlServerModels<B>,
    #[builder(default)]
    pub(crate) server_state: ServerState,
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
                if let Some(ref payloads) = rx_payloads {
                    let mut units = filter_units_to_fwd(target_state, payloads);
                    self.fl_models
                        .composer
                        .append_units_to(&mut this_payload, &mut units);
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
        mut target_classes: Vec<AgentClass>,
        message_to_build: FlMessageToBuild,
        target_agents: Option<Vec<AgentId>>,
    ) {
        target_classes.iter_mut().for_each(|target_class| {
            if let Some(links) = self.get_links(target_class, bucket) {
                self.fl_models
                    .composer
                    .set_message_to_build(message_to_build);
                let payload =
                    self.fl_models
                        .composer
                        .compose(self.step, target_class, &self.server_info);
                let mut actions = self.models.actor.actions_for(target_class).to_owned();

                if let Some(agents) = &target_agents {
                    actions
                        .values_mut()
                        .for_each(|action| action.update_broadcast(agents));
                }
                debug!("Actions from server: {:?}", actions);
                self.send_payload(links, target_class, payload, &None, bucket, actions);
            }
        });
    }

    fn do_fl_actions(&mut self, bucket: &mut FlBucket<B>, payloads: &mut Vec<FlPayload>) {
        for payload in payloads.iter_mut() {
            debug!("Server Payload is {:?} at {}", payload, self.step);
            let mut client_infos = Vec::new();
            client_infos.push(payload.agent_state);
            client_infos.extend(payload.gathered_states.clone());
            payload
                .data_units
                .iter_mut()
                .zip(client_infos.iter())
                .for_each(|(data_unit, client_info)| match data_unit.fl_content {
                    FlContent::None => {}
                    FlContent::StateInfo => self.register_client(client_info),
                    FlContent::LocalModel => self.collect_local_model(bucket, client_info),
                    FlContent::TrainingFailed => debug!("a client failed to train"),
                    _ => panic!("Server should not receive this message"),
                });
            payload
                .data_units
                .iter_mut()
                .for_each(|message| message.action.action_type = ActionType::Consume);
        }
    }

    fn register_client(&mut self, agent_info: &AgentInfo) {
        if self
            .fl_models
            .client_classes
            .contains(&agent_info.agent_class)
        {
            debug!("Registering client from agent {}", agent_info.id);
            self.fl_models.client_selector.register_client(agent_info);
        }
    }

    fn collect_local_model(&mut self, bucket: &mut FlBucket<B>, client_info: &AgentInfo) {
        debug!(
            "Collecting local models at {} from {}",
            self.step, client_info.id
        );
        let local_model = bucket.models.model_lake.local_model_of(client_info.id);
        self.fl_models.aggregator.add_local_model(local_model);
        debug!("Finished collecting local model at {}", self.step);
    }

    fn handle_initiation(&mut self, bucket: &mut FlBucket<B>, target_classes: Vec<AgentClass>) {
        debug!("Changing from idle to analysis at {}", self.step);
        self.server_state = ServerState::ClientAnalysis;

        let message_to_build = FlMessageToBuild::builder()
            .message(Message::FlMessage)
            .message_type(MessageType::KiloByte)
            .quantity(1)
            .fl_content(FlContent::StateInfo)
            .build();
        self.send_fl_message(bucket, target_classes, message_to_build, None);
    }

    fn handle_analysis(&mut self, bucket: &mut FlBucket<B>, target_classes: Vec<AgentClass>) {
        let mut selected_clients = None;
        let mut message_to_build = FlMessageToBuild::builder()
            .message(Message::FlMessage)
            .message_type(MessageType::KiloByte)
            .quantity(1)
            .fl_content(FlContent::StateInfo)
            .build();

        if self.fl_models.client_selector.has_clients() {
            self.fl_models.client_selector.do_selection();

            if self.fl_models.client_selector.selected_clients().len() > 0 {
                debug!("Changing from analysis to selection at {}", self.step);
                self.fl_models
                    .times
                    .update_time(self.step, self.server_state);

                self.server_state = ServerState::ClientSelection;

                selected_clients =
                    Some(self.fl_models.client_selector.selected_clients().to_owned());

                message_to_build = FlMessageToBuild::builder()
                    .message(Message::FlMessage)
                    .message_type(MessageType::KiloByte)
                    .quantity(1)
                    .fl_content(FlContent::ClientSelected)
                    .build();
            }
        } else {
            debug!(
                "No clients registered. Continuing with analysis at {}",
                self.step
            );
        }

        self.send_fl_message(bucket, target_classes, message_to_build, selected_clients);
    }

    fn handle_selection(&mut self, bucket: &mut FlBucket<B>, target_classes: Vec<AgentClass>) {
        debug!("Changing from selection to training at {}", self.step);
        self.server_state = ServerState::TrainingRound;

        let message_to_build = FlMessageToBuild::builder()
            .message(Message::FlMessage)
            .message_type(MessageType::KiloByte)
            .quantity(1)
            .fl_content(FlContent::GlobalModel)
            .build();
        let selected_clients = self.fl_models.client_selector.selected_clients().to_owned();
        debug!("Selected clients are: {:?}", selected_clients);
        self.send_fl_message(
            bucket,
            target_classes,
            message_to_build,
            Some(selected_clients),
        );
    }

    fn handle_training(&mut self, bucket: &mut FlBucket<B>, target_classes: Vec<AgentClass>) {
        debug!("Changing from training to aggregation at {}", self.step);
        self.server_state = ServerState::Aggregation;

        let message_to_build = FlMessageToBuild::builder()
            .message(Message::FlMessage)
            .message_type(MessageType::KiloByte)
            .quantity(1)
            .fl_content(FlContent::CompleteTraining)
            .build();
        let selected_clients = self.fl_models.client_selector.selected_clients().to_owned();
        self.send_fl_message(
            bucket,
            target_classes,
            message_to_build,
            Some(selected_clients),
        );
    }

    fn handle_aggregation(&mut self, bucket: &mut FlBucket<B>, target_classes: Vec<AgentClass>) {
        debug!("Changing from aggregation to idle at {}", self.step);
        self.server_state = ServerState::Idle;

        debug!("Aggregating local models to global model");
        let current_global_model = self.fl_models.trainer.model.clone();
        self.fl_models.trainer.model = self
            .fl_models
            .aggregator
            .aggregate(current_global_model, &bucket.models.device);

        debug!("Writing model to file ");
        self.fl_models.trainer.save_model_to_file(self.step);
        bucket
            .models
            .model_lake
            .update_global_model(self.fl_models.trainer.model.clone(), self.step);

        let model_accuracy = self.fl_models.trainer.test_model(&bucket.models.device);
        info!(
            "Global model accuracy is {} at {}",
            model_accuracy, self.step
        );

        let message_to_build = FlMessageToBuild::builder()
            .message(Message::FlMessage)
            .message_type(MessageType::KiloByte)
            .quantity(1)
            .fl_content(FlContent::StateInfo)
            .build();
        let selected_clients = self.fl_models.client_selector.selected_clients().to_owned();
        self.send_fl_message(
            bucket,
            target_classes,
            message_to_build,
            Some(selected_clients),
        );
    }
}

impl<B: AutodiffBackend> Activatable<FlBucket<B>> for Server<B> {
    fn activate(&mut self, bucket: &mut FlBucket<B>) {
        debug!("Starting agent: {}", self.server_info.id);
        self.power_state = PowerState::On;

        bucket.update_agent_data_of(self.server_info.id, self.server_info);
        self.fl_models
            .times
            .update_time(self.step, self.server_state);

        self.fl_models.trainer.test_data = bucket.models.data_distributor.get_server_test_data();

        debug!(
            "Test data server length {:?}",
            self.fl_models.trainer.test_data.length()
        );

        self.fl_models
            .holder
            .set_test_data(self.fl_models.trainer.test_data.to_owned());

        self.fl_models.holder.allot_data(self.server_info.id);

        bucket.models.model_lake.global_model = Some(self.fl_models.trainer.model.clone());
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
            "Transmitting payload from agent {} to agent {} with units {:?}",
            payload.agent_state.id(),
            target.target,
            payload.data_units,
        );

        self.models.flow.register_outgoing_attempt(&payload);
        let tx_metrics = bucket.models.network.transfer(&payload);
        if let Some(tx) = &mut bucket.models.output.tx_data_writer {
            tx.add_data(self.step, &target, &payload, tx_metrics)
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

        if let Some(tx) = &mut bucket.models.output.tx_data_writer {
            tx.add_data(self.step, &target, &payload, sl_metrics)
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

        if let Some(ref mut payloads) = rx_payloads {
            debug!("S.Received payloads length: {}", payloads.len());
            payloads.clone().iter().for_each(|p| {
                debug!(
                    "S.Message of type: {:?}, from {}, with units: {:?}",
                    p.query_type, p.agent_state.id, p.data_units,
                );
            });
            self.do_fl_actions(bucket, payloads);
            self.models.flow.register_incoming(payloads);
            payloads.iter_mut().for_each(|payload| {
                complete_actions(payload, &self.server_info);
            });
        }
    }

    fn stage_two_reverse(&mut self, bucket: &mut FlBucket<B>) {
        debug!(
            "Downlink stage for agent: {} id at step: {}",
            self.server_info.id, self.step
        );

        let target_classes = self.models.directions.stage_two.target_classes.clone();

        // Depending on the training state, send messages to clients.
        debug!("Time to change is {}", self.fl_models.times.next_change_at);
        if self.fl_models.times.is_time_to_change(self.step) {
            debug!("Changing server state at {}", self.step);
            match self.server_state {
                ServerState::Idle => self.handle_initiation(bucket, target_classes),
                ServerState::ClientAnalysis => self.handle_analysis(bucket, target_classes),
                ServerState::ClientSelection => self.handle_selection(bucket, target_classes),
                ServerState::TrainingRound => self.handle_training(bucket, target_classes),
                ServerState::Aggregation => self.handle_aggregation(bucket, target_classes),
            }
            self.fl_models
                .times
                .update_time(self.step, self.server_state);
        } else {
            let mut message_to_build = FlMessageToBuild::builder()
                .message(Message::FlMessage)
                .message_type(MessageType::KiloByte)
                .quantity(1)
                .fl_content(FlContent::None)
                .build();
            let mut target_agents = None;

            match self.server_state {
                ServerState::Idle => {}
                ServerState::ClientAnalysis => message_to_build.fl_content = FlContent::StateInfo,
                ServerState::ClientSelection => {
                    message_to_build.message_type = MessageType::KiloByte;
                    message_to_build.fl_content = FlContent::ClientSelected;
                    target_agents = Some(self.fl_models.client_selector.selected_clients().clone());
                }
                ServerState::TrainingRound => {}
                ServerState::Aggregation => {}
            };
            self.send_fl_message(bucket, target_classes, message_to_build, target_agents);
        }

        // talk to the agents in side link.
        if self.models.directions.stage_two.is_sidelink {
            let target_class = self.server_info.agent_class.clone();
            if let Some(links) = self.get_links(&target_class, bucket) {
                let payload =
                    self.fl_models
                        .composer
                        .compose(self.step, &target_class, &self.server_info);
                let actions = self.models.actor.actions_for(&target_class).to_owned();
                self.send_payload(links, &target_class, payload, &None, bucket, actions);
            }
        }
    }

    fn stage_three(&mut self, bucket: &mut FlBucket<B>) {
        // Receive data from the peers.
        self.receive_sl(bucket);
    }

    fn stage_four_reverse(&mut self, bucket: &mut FlBucket<B>) {
        bucket.models.output.basic_results.rx_counts.add_data(
            self.step,
            self.server_info.id,
            &self.models.flow.comm_stats.outgoing_stats,
        );

        if let Some(writer) = &mut bucket.models.output.fl_state_writer {
            writer.add_data(self.step, self.server_info.id, self.server_state.value())
        }

        if self.step == self.models.power.peek_time_to_off() {
            self.power_state = PowerState::Off;
        }
    }

    fn stage_five(&mut self, _bucket: &mut FlBucket<B>) {
        self.stats = self.models.flow.comm_stats;
    }
}
