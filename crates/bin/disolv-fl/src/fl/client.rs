use burn::backend::wgpu::WgpuDevice;
use burn::prelude::Backend;
use burn::tensor::backend::AutodiffBackend;
use log::debug;
use typed_builder::TypedBuilder;

use disolv_core::agent::{
    Activatable, Agent, AgentClass, AgentId, AgentKind, AgentOrder, AgentProperties, Movable,
    Orderable,
};
use disolv_core::bucket::TimeMS;
use disolv_core::hashbrown::HashMap;
use disolv_core::metrics::Bytes;
use disolv_core::radio::{Action, ActionType, Link, Receiver, Transmitter};
use disolv_models::device::actions::{
    am_i_target, complete_actions, filter_units_to_fwd, set_actions_before_tx,
};
use disolv_models::device::actor::Actor;
use disolv_models::device::directions::Directions;
use disolv_models::device::flow::FlowRegister;
use disolv_models::device::metrics::MegaHertz;
use disolv_models::device::mobility::MapState;
use disolv_models::device::models::{Compose, LinkSelect};
use disolv_models::device::power::{PowerManager, PowerState};
use disolv_models::net::metrics::Bandwidth;
use disolv_models::net::radio::{CommStats, LinkProperties};

use crate::fl::bucket::FlBucket;
use crate::models::ai::compose::{FlComposer, FlMessageToBuild};
use crate::models::ai::data::DataHolder;
use crate::models::ai::mnist::mnist_train;
use crate::models::ai::models::{ClientState, DatasetType, ModelType};
use crate::models::ai::times::ClientTimes;
use crate::models::ai::trainer::Trainer;
use crate::models::device::energy::EnergyType;
use crate::models::device::hardware::Hardware;
use crate::models::device::link::LinkSelector;
use crate::models::device::message::{
    FlContent, FlPayload, FlPayloadInfo, Message, MessageType, MessageUnit,
};

#[derive(Clone, Copy, Debug, Default, TypedBuilder)]
pub(crate) struct AgentInfo {
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
    pub(crate) actor: Actor<MessageType>,
    pub(crate) energy: EnergyType,
    pub(crate) hardware: Hardware,
    pub(crate) directions: Directions,
    pub(crate) selector: Vec<(AgentClass, LinkSelector)>,
}

#[derive(Clone, TypedBuilder)]
pub(crate) struct FlModels<B: AutodiffBackend> {
    pub(crate) trainer: Trainer<B>,
    pub(crate) local_model: ModelType<B>,
    pub(crate) composer: FlComposer,
    pub(crate) times: ClientTimes,
    pub(crate) holder: DataHolder,
}

#[derive(Clone, TypedBuilder)]
pub(crate) struct Client<B: AutodiffBackend> {
    pub(crate) client_info: AgentInfo,
    pub(crate) models: ClientModels,
    pub(crate) fl_models: FlModels<B>,
    #[builder(default)]
    pub(crate) client_state: ClientState,
    #[builder(default)]
    pub(crate) step: TimeMS,
    #[builder(default)]
    pub(crate) power_state: PowerState,
    #[builder(default)]
    pub(crate) map_state: MapState,
    #[builder(default)]
    pub(crate) stats: CommStats,
}

impl<B: AutodiffBackend> Client<B> {
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

    fn intended_target(
        &self,
        actions: &HashMap<MessageType, Action>,
        target_link: Link<LinkProperties>,
    ) -> bool {
        let mut result = true;
        actions.values().take(0).for_each(|action| {
            if let Some(agent_id) = action.to_agent.clone() {
                result = agent_id == target_link.target;
            }

            if let Some(broadcast_vec) = action.to_broadcast.clone() {
                result = broadcast_vec.contains(&target_link.target);
            }
        });
        result
    }

    fn get_links(
        &mut self,
        target_class: &AgentClass,
        bucket: &mut FlBucket<B>,
    ) -> Option<Vec<Link<LinkProperties>>> {
        let link_options = match bucket.link_options_for(
            self.client_info.id,
            &self.client_info.agent_type,
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

                    payloads.iter().for_each(|rx_payload| {
                        this_payload.gathered_states.push(
                            *bucket
                                .agent_data_of(&rx_payload.agent_state.id)
                                .expect("unable to find agent"),
                        )
                    });
                }
            }

            let prepared_payload = set_actions_before_tx(this_payload, &actions);
            if target_class == &self.client_info.agent_class {
                self.transmit_sl(prepared_payload, target_link, bucket);
            } else {
                self.transmit(prepared_payload, target_link, bucket);
            }
        });
    }

    fn send_fl_message(
        &mut self,
        bucket: &mut FlBucket<B>,
        rx_payloads: &Option<Vec<FlPayload>>,
        mut target_classes: Vec<AgentClass>,
    ) {
        target_classes.iter_mut().for_each(|target_class| {
            if let Some(links) = self.get_links(target_class, bucket) {
                let payload =
                    self.fl_models
                        .composer
                        .compose(self.step, target_class, &self.client_info);
                let actions = self.models.actor.actions_for(target_class).to_owned();
                self.send_payload(links, target_class, payload, rx_payloads, bucket, actions);
            }
        });
    }

    fn do_fl_actions(&mut self, bucket: &mut FlBucket<B>, payloads: &mut Vec<FlPayload>) {
        for payload in payloads.iter_mut() {
            debug!("Processing payload of type {:?}", payload.query_type);
            payload.data_units.iter_mut().for_each(|message_unit| {
                if message_unit.action.action_type == ActionType::Consume {
                    match message_unit.fl_content {
                        FlContent::None => {}
                        FlContent::StateInfo => self.prepare_state_update(),
                        FlContent::GlobalModel => self.collect_global_model(bucket),
                        FlContent::ClientSelected => self.initiate_training(bucket),
                        FlContent::CompleteTraining => self.complete_training(bucket),
                        _ => panic!("Client should not receive this message"),
                    }
                    message_unit.action.action_type = ActionType::Consume;
                }
            });
        }
    }

    fn prepare_state_update(&mut self) {
        debug!(
            "0. Agent {} state at {} is {:?}",
            self.client_info.id, self.step, self.client_state
        );
        if self.client_state == ClientState::Waiting {
            return;
        }
        self.client_state = ClientState::Waiting;

        debug!(
            "Preparing to send a state update to server in agent {} at {}",
            self.client_info.id, self.step
        );

        let message_to_send = FlMessageToBuild::builder()
            .message(Message::FlMessage)
            .message_type(MessageType::KiloByte)
            .quantity(1)
            .fl_content(FlContent::StateInfo)
            .build();
        self.fl_models
            .composer
            .set_message_to_build(message_to_send);
    }

    fn collect_global_model(&mut self, bucket: &mut FlBucket<B>) {
        debug!(
            "1. Agent {} state at {} is {:?}",
            self.client_info.id, self.step, self.client_state
        );
        self.fl_models.local_model = match bucket.models.model_lake.global_model.to_owned() {
            Some(val) => val,
            None => panic!("Global model not present"),
        };
        debug!(
            "Updating local model with built model at {}",
            bucket.models.model_lake.update_time()
        );
    }

    fn initiate_training(&mut self, bucket: &mut FlBucket<B>) {
        debug!(
            "2. Agent {} state at {} is {:?}",
            self.client_info.id, self.step, self.client_state
        );
        if self.client_state == ClientState::Training {
            return;
        }
        self.client_state = ClientState::Training;

        self.fl_models
            .times
            .update_time(self.step, self.client_state);

        self.fl_models.trainer.train_data = self.fl_models.holder.allotted_training_data();
        self.fl_models.trainer.train_data = self.fl_models.holder.allotted_testing_data();

        debug!(
            "Starting local training in agent {} at {} with input size {}",
            self.client_info.id,
            self.step,
            self.fl_models.trainer.train_data.length()
        );

        self.fl_models.trainer.train(&bucket.models.device);
        self.fl_models.trainer.save_model_to_file(self.step);
        self.fl_models.local_model = self.fl_models.trainer.model.to_owned();
    }

    fn complete_training(&mut self, bucket: &mut FlBucket<B>) {
        debug!(
            "3. Agent {} state at {} is {:?}",
            self.client_info.id, self.step, self.client_state
        );
        if self.client_state == ClientState::Sensing {
            return;
        }
        self.client_state = ClientState::Sensing;

        debug!("Completed local training at {}", self.step);
        let mut message_to_send = FlMessageToBuild::default();

        if self.is_training_complete() {
            debug!("Sending local model");
            message_to_send.message = Message::FlMessage;
            message_to_send.fl_content = FlContent::LocalModel;
            message_to_send.message_type = MessageType::F64Weights;
            message_to_send.quantity = self.fl_models.trainer.no_of_weights;
            bucket
                .models
                .model_lake
                .add_local_model(self.client_info.id, self.fl_models.trainer.model.clone());
        } else {
            debug!("Local training failed");
            message_to_send.message = Message::FlMessage;
            message_to_send.fl_content = FlContent::TrainingFailed;
            message_to_send.message_type = MessageType::KiloByte;
            message_to_send.quantity = 1;
        }

        self.fl_models
            .composer
            .set_message_to_build(message_to_send);
    }

    fn is_training_complete(&self) -> bool {
        // todo: add dynamic training times considering hardware usage etc.
        debug!(
            "Step now: {}, {}",
            self.step, self.fl_models.times.next_change_at
        );
        self.step > self.fl_models.times.next_change_at
    }
}

impl<B: AutodiffBackend> Activatable<FlBucket<B>> for Client<B> {
    fn activate(&mut self, bucket: &mut FlBucket<B>) {
        self.power_state = PowerState::On;
        bucket.update_agent_data_of(self.client_info.id, self.client_info);

        self.fl_models.trainer.train_data = bucket
            .training_data_for(self.client_info.id)
            .expect("no training data set for this agent");
        self.fl_models.trainer.test_data = bucket
            .testing_data_for(self.client_info.id)
            .expect("no test data set for this agent");

        debug!(
            "Train data {:?} in agent {}",
            self.fl_models.trainer.train_data.length(),
            self.client_info.id,
        );
        debug!(
            "Test data {:?} in agent {}",
            self.fl_models.trainer.test_data.length(),
            self.client_info.id,
        );

        self.fl_models
            .holder
            .set_train_data(self.fl_models.trainer.train_data.to_owned());
        self.fl_models
            .holder
            .set_test_data(self.fl_models.trainer.test_data.to_owned());
        // debug!(
        //     "Client {} has training size {} and testing size {}",
        //     self.client_info.id,
        //     self.fl_models.trainer.train_data.length(),
        //     self.fl_models.trainer.test_data.length()
        // );
        self.client_state = ClientState::Sensing;
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

impl<B: AutodiffBackend> Orderable for Client<B> {
    fn order(&self) -> AgentOrder {
        self.client_info.agent_order
    }
}

impl<B: AutodiffBackend> Movable<FlBucket<B>> for Client<B> {
    type M = MapState;

    fn mobility(&self) -> &Self::M {
        &self.map_state
    }

    fn set_mobility(&mut self, bucket: &mut FlBucket<B>) {
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

impl<B: AutodiffBackend>
    Transmitter<
        FlBucket<B>,
        MessageType,
        MessageUnit,
        LinkProperties,
        FlPayloadInfo,
        AgentInfo,
        Message,
    > for Client<B>
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
            payload.data_units
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
    for Client<B>
{
    fn receive(&mut self, bucket: &mut FlBucket<B>) -> Option<Vec<FlPayload>> {
        bucket.models.data_lake.payloads_for(self.client_info.id)
    }

    fn receive_sl(&mut self, bucket: &mut FlBucket<B>) -> Option<Vec<FlPayload>> {
        bucket.models.data_lake.sl_payloads_for(self.client_info.id)
    }
}

impl<B: AutodiffBackend> Agent<FlBucket<B>> for Client<B> {
    fn id(&self) -> AgentId {
        self.client_info.id
    }

    fn stage_one(&mut self, bucket: &mut FlBucket<B>) {
        self.step = bucket.step;
        self.set_mobility(bucket);
        self.fl_models.holder.allot_data(self.client_info.id);

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

        let target_classes = self.models.directions.stage_one.target_classes.clone();
        self.send_fl_message(bucket, &rx_payloads, target_classes);

        if !self.models.directions.stage_one.is_sidelink {
            return;
        }

        let target_class = self.client_info.agent_class.clone();
        if let Some(links) = self.get_links(&target_class, bucket) {
            let payload =
                self.fl_models
                    .composer
                    .compose(self.step, &target_class, &self.client_info);
            let actions = self.models.actor.actions_for(&target_class).to_owned();
            self.send_payload(links, &target_class, payload, &rx_payloads, bucket, actions);
        }
    }

    fn stage_two_reverse(&mut self, bucket: &mut FlBucket<B>) {
        debug!(
            "Downlink stage for agent: {} id at step: {}",
            self.client_info.id, self.step
        );

        let mut rx_payloads = self.receive(bucket);

        if let Some(ref mut payloads) = rx_payloads {
            debug!("Received payloads length: {}", payloads.len());
            payloads.clone().iter().for_each(|p| {
                debug!(
                    "Message of type: {:?}, from {}, with units: {:?}",
                    p.query_type, p.agent_state.id, p.data_units,
                );
            });
            self.do_fl_actions(bucket, payloads);
            self.models.flow.register_incoming(payloads);
            payloads.iter_mut().for_each(|payload| {
                complete_actions(payload, &self.client_info);
            });
        }

        let target_classes = self.models.directions.stage_two.target_classes.clone();
        self.send_fl_message(bucket, &rx_payloads, target_classes);
    }

    fn stage_three(&mut self, bucket: &mut FlBucket<B>) {}

    fn stage_four_reverse(&mut self, bucket: &mut FlBucket<B>) {
        debug!(
            "Last stage for agent: {} id at step: {}",
            self.client_info.id, self.step
        );

        bucket.models.output.basic_results.rx_counts.add_data(
            self.step,
            self.client_info.id,
            &self.models.flow.comm_stats.outgoing_stats,
        );

        if let Some(writer) = &mut bucket.models.output.fl_state_writer {
            writer.add_data(self.step, self.client_info.id, self.client_state.value())
        }

        if self.step == self.models.power.peek_time_to_off() {
            self.power_state = PowerState::Off;
        }
    }
}
