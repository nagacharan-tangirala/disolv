use crate::actions::{assign_actions, do_actions};
use crate::latency::{LatencyConfig, LatencyType};
use log::{debug, error};
use pavenet_core::entity::NodeClass;
use pavenet_core::message::RxMetrics;
use pavenet_core::message::{DPayload, DataType, NodeContent, PayloadInfo, RxFailReason, RxStatus};
use pavenet_core::radio::{ActionImpl, ActionSettings, InDataStats, OutDataStats};
use pavenet_core::rand_pcg::Pcg64Mcg;
use pavenet_engine::bucket::TimeMS;
use pavenet_engine::hashbrown::HashMap;
use pavenet_engine::metrics::{Feasibility, Measurable};
use pavenet_engine::node::NodeId;
use pavenet_engine::radio::{Channel, IncomingStats, OutgoingStats, SlChannel};
use rand::prelude::SliceRandom;
use serde::Deserialize;
use typed_builder::TypedBuilder;

#[derive(Deserialize, Debug, Clone)]
pub struct RxSettings {
    pub latency: LatencyConfig,
}

#[derive(Clone, Debug, TypedBuilder)]
pub struct RadioModels {
    pub latency_type: LatencyType,
}

#[derive(Clone, Debug, TypedBuilder)]
pub struct Radio {
    pub step_size: TimeMS,
    pub rng: Pcg64Mcg,
    pub models: RadioModels,
    #[builder(default)]
    pub rx_payloads: Vec<DPayload>,
    #[builder(default)]
    pub rx_metrics: Vec<RxMetrics>,
    #[builder(default)]
    pub in_link_nodes: HashMap<NodeClass, Vec<NodeId>>,
    #[builder(default)]
    pub in_stats: InDataStats,
    #[builder(default)]
    pub actions: HashMap<NodeClass, HashMap<DataType, ActionImpl>>,
    #[builder(default)]
    pub out_link_nodes: HashMap<NodeClass, Vec<NodeId>>,
    #[builder(default)]
    pub out_stats: OutDataStats,
}

impl Radio {
    pub fn update_settings(&mut self, action_settings: &Vec<ActionSettings>) {
        let mut actions = HashMap::with_capacity(action_settings.len());
        for action in action_settings.iter() {
            let rule = ActionImpl::builder()
                .action_type(action.action_type)
                .to_kind(action.to_kind)
                .to_class(action.to_class)
                .to_node(action.to_node)
                .build();

            actions
                .entry(action.target)
                .or_insert(HashMap::new())
                .entry(action.data_type)
                .or_insert(rule);
        }
        self.actions = actions;
    }

    fn measure_rx(&mut self, payloads: &Vec<DPayload>) {
        let mut rx_order = 1;
        for payload in payloads.iter() {
            let mut rx_stat = RxMetrics::new(payload, rx_order);
            match self
                .models
                .latency_type
                .measure(&rx_stat, &payload.metadata)
            {
                Feasibility::Feasible(latency) => rx_stat.latency = latency,
                Feasibility::Infeasible(latency) => {
                    rx_stat.latency = latency;
                    rx_stat.rx_status = RxStatus::Fail;
                    rx_stat.rx_fail_reason = RxFailReason::LatencyLimit;
                    self.rx_metrics.push(rx_stat);
                    rx_order += 1;
                    continue;
                }
            };
            rx_stat.rx_status = RxStatus::Ok;
            self.rx_metrics.push(rx_stat);
            rx_order += 1;
        }
    }

    pub fn transfer_stats(&mut self) -> Vec<RxMetrics> {
        self.rx_metrics.clone()
    }
}

impl Channel<PayloadInfo, NodeContent> for Radio {
    type C = NodeClass;
    fn reset(&mut self) {
        self.rx_metrics.clear();
        self.rx_payloads.clear();
        self.in_stats.reset();
        self.in_link_nodes.clear();
        self.out_stats.reset();
        self.out_link_nodes.clear();
    }

    fn prepare_transfer(&mut self, target_class: &NodeClass, mut payload: DPayload) -> DPayload {
        debug!(
            "Preparing payload with {} data blobs for transfer to class {}",
            payload.metadata.data_blobs.len(),
            target_class
        );
        self.out_stats.update(&payload.metadata);
        self.out_link_nodes
            .entry(payload.node_state.node_info.node_class)
            .or_default()
            .push(payload.node_state.node_info.id);

        let actions = match self.actions.get(target_class) {
            Some(acts) => acts,
            None => return payload,
        };
        payload.metadata.data_blobs.iter_mut().for_each(|blob| {
            let new_action = match actions.get(&blob.data_type) {
                Some(action) => action,
                None => {
                    error!(
                        "No action found for data type {} in class {}",
                        blob.data_type, target_class
                    );
                    panic!("Action missing for data type {}", blob.data_type);
                }
            };
            assign_actions(blob, new_action);
        });
        payload
    }

    fn do_receive(&mut self, node_info: &NodeContent, mut payloads: Vec<DPayload>) {
        payloads.shuffle(&mut self.rng);
        self.measure_rx(&payloads);
        payloads
            .into_iter()
            .zip(self.rx_metrics.iter())
            .for_each(|(payload, rx_stat)| {
                self.in_stats.add_attempted(&payload.metadata);
                self.in_link_nodes
                    .entry(payload.node_state.node_info.node_class)
                    .or_insert(Vec::new())
                    .push(payload.node_state.node_info.id);

                if rx_stat.rx_status == RxStatus::Ok {
                    self.in_stats.add_feasible(&payload.metadata);
                    self.rx_payloads.push(payload.clone());
                }
            });

        self.rx_payloads
            .iter_mut()
            .for_each(|payload| do_actions(payload, node_info));
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct SlSettings {
    pub latency: LatencyConfig,
}

#[derive(Clone, Debug, TypedBuilder)]
pub struct SlRadio {
    pub step_size: TimeMS,
    pub rng: Pcg64Mcg,
    pub models: RadioModels,
    #[builder(default)]
    pub sl_payloads: Vec<DPayload>,
    #[builder(default)]
    pub in_link_nodes: Vec<NodeId>,
    #[builder(default)]
    pub in_stats: InDataStats,
    #[builder(default)]
    pub sl_metrics: Vec<RxMetrics>,
    #[builder(default)]
    pub actions: HashMap<DataType, ActionImpl>,
    #[builder(default)]
    pub out_link_nodes: Vec<NodeId>,
    #[builder(default)]
    pub out_stats: OutDataStats,
}

impl SlRadio {
    pub fn update_settings(&mut self, action_settings: &Vec<ActionSettings>) {
        let mut actions = HashMap::with_capacity(action_settings.len());
        for action in action_settings.iter() {
            let rule = ActionImpl::builder()
                .action_type(action.action_type)
                .to_kind(action.to_kind)
                .to_class(action.to_class)
                .to_node(action.to_node)
                .build();

            actions
                .entry(action.target)
                .or_insert(HashMap::new())
                .entry(action.data_type)
                .or_insert(rule);
        }
        self.actions = actions;
    }

    /// Sets the new action for the data blob
    ///
    /// # Arguments
    /// * `data_blob` - The data blob to set the action for
    /// * `new_action` - The new action to set
    fn assign_actions(&self, data_blob: &mut DataBlob, new_action: &ActionImpl) {
        match new_action.action_type {
            ActionType::Consume => {
                data_blob.action.action_type = ActionType::Consume;
            }
            ActionType::Forward => {
                if let Some(target_node) = new_action.to_node {
                    data_blob.action.to_node = Some(target_node);
                }
                if let Some(target_class) = new_action.to_class {
                    data_blob.action.to_class = Some(target_class);
                }
                if let Some(target_kind) = new_action.to_kind {
                    data_blob.action.to_kind = Some(target_kind);
                }
                data_blob.action.action_type = ActionType::Forward;
            }
        };
    }

    /// Checks if the current node should forward the data blob
    ///
    /// # Arguments
    /// * `blob` - The data blob to check
    /// * `target_info` - The node info of the target node
    ///
    /// # Returns
    /// * `bool` - True if the current node should forward the data blob, false otherwise
    fn should_i_forward(&self, blob: &DataBlob, target_info: &NodeInfo) -> bool {
        if blob.action.action_type == ActionType::Consume {
            error!("This should have been consumed by now");
            panic!("consume payload appears to be forwarded");
        }
        if let Some(target_id) = blob.action.to_node {
            if target_id == target_info.id {
                return true;
            }
        }
        if let Some(class) = blob.action.to_class {
            if class == target_info.node_class {
                return true;
            }
        }
        if let Some(target_kind) = blob.action.to_kind {
            if target_info.node_type == target_kind {
                return true;
            }
        }
        false
    }
}

impl TxChannel<PayloadInfo, NodeContent> for TxRadio {
    type C = NodeClass;
    type D = DataBlob;

    fn reset(&mut self) {
        self.out_stats.reset();
        self.out_link_nodes.clear();
    }

    fn prepare_blobs_to_fwd(
        &mut self,
        target_node_data: &NodeContent,
        to_forward: &Vec<DPayload>,
    ) -> Vec<DataBlob> {
        let mut blobs_to_forward: Vec<DataBlob> = Vec::new();
        for payload in to_forward.iter() {
            for blob in payload.metadata.data_blobs.iter() {
                if self.should_i_forward(blob, &target_node_data.node_info) {
                    blobs_to_forward.push(blob.to_owned());
                }
            }
        }
        blobs_to_forward
    }

    fn prepare_transfer(&mut self, target_class: &NodeClass, mut payload: DPayload) -> DPayload {
        self.out_stats.update(&payload.metadata);
        self.out_link_nodes
            .entry(payload.node_state.node_info.node_class)
            .or_default()
            .push(payload.node_state.node_info.id);

        let actions = match self.actions.get(target_class) {
            Some(acts) => acts,
            None => return payload,
        };
        payload.metadata.data_blobs.iter_mut().for_each(|blob| {
            let new_action = match actions.get(&blob.data_type) {
                Some(action) => action,
                None => {
                    error!(
                        "No action found for data type {} in class {}",
                        blob.data_type, target_class
                    );
                    panic!("Action missing for data type {}", blob.data_type);
                }
            };
            self.assign_actions(blob, new_action);
        });
        payload
    }
}
