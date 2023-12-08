use crate::latency::LatencyType;
use log::error;
use pavenet_core::entity::{NodeClass, NodeInfo};
use pavenet_core::message::{DPayload, DataType, NodeContent, PayloadInfo, RxFailReason, RxStatus};
use pavenet_core::message::{DataBlob, RxMetrics};
use pavenet_core::radio::{ActionImpl, ActionSettings, ActionType, InDataStats, OutDataStats};
use pavenet_core::rand_pcg::Pcg64Mcg;
use pavenet_engine::bucket::TimeMS;
use pavenet_engine::hashbrown::HashMap;
use pavenet_engine::metrics::{Feasibility, Measurable};
use pavenet_engine::node::NodeId;
use pavenet_engine::radio::{IncomingStats, OutgoingStats, RxChannel, TxChannel};
use rand::prelude::SliceRandom;
use typed_builder::TypedBuilder;

#[derive(Clone, Debug, TypedBuilder)]
pub struct RxRadio {
    pub latency_type: LatencyType,
    pub step_size: TimeMS,
    pub rng: Pcg64Mcg,
    #[builder(default)]
    pub in_link_nodes: HashMap<NodeClass, Vec<NodeId>>,
    #[builder(default)]
    pub in_stats: InDataStats,
    #[builder(default)]
    pub rx_metrics: HashMap<NodeId, RxMetrics>,
}

impl RxRadio {
    fn measure_rx(&mut self, rx_order: u32, payload: &DPayload) -> RxMetrics {
        let mut rx_stats = RxMetrics::new(payload.node_state.node_info.id, rx_order);
        match self.latency_type.measure(&rx_stats, &payload.metadata) {
            Feasibility::Feasible(latency) => rx_stats.latency = latency,
            Feasibility::Infeasible(latency) => {
                rx_stats.latency = latency;
                rx_stats.rx_status = RxStatus::Fail;
                rx_stats.rx_fail_reason = RxFailReason::LatencyLimit;
                return rx_stats;
            }
        };
        rx_stats.rx_status = RxStatus::Ok;
        rx_stats
    }

    pub fn transfer_stats(&mut self) -> HashMap<NodeId, RxMetrics> {
        self.rx_metrics.clone()
    }

    /// Performs the actions instructed by the sender.
    /// At this point, we have received the data blobs with actions instructed by the sender.
    /// We need to apply these actions to the data blobs and set the actions for the next hop.
    ///
    /// # Arguments
    /// * `payload` - The payload to set actions for
    /// * `node_info` - The node info of the current node
    ///
    /// # Returns
    /// * `DPayload` - The payload with the new actions set
    fn do_actions(&self, mut payload: DPayload, node_content: &NodeContent) -> DPayload {
        payload
            .metadata
            .data_blobs
            .iter_mut()
            .for_each(|blob| match blob.action.action_type {
                ActionType::Consume => {}
                ActionType::Forward => {
                    if self.am_i_target(&blob.action, &node_content.node_info) {
                        blob.action.action_type = ActionType::Consume;
                    }
                }
            });
        payload.metadata.consume();
        payload
    }

    /// Checks if the current node is the intended target of the data
    ///
    /// # Arguments
    /// * `action` - The action to check
    /// * `node_info` - The node info of the current node
    ///
    /// # Returns
    /// * `bool` - True if the current node is the intended target, false otherwise
    fn am_i_target(&self, action: &ActionImpl, node_info: &NodeInfo) -> bool {
        // Order of precedence: Node -> Class -> Kind
        if let Some(target_node) = action.to_node {
            if target_node == node_info.id {
                return true;
            }
        }
        if let Some(target_class) = action.to_class {
            if target_class == node_info.node_class {
                return true;
            }
        }
        if let Some(target_kind) = action.to_kind {
            if target_kind == node_info.node_type {
                return true;
            }
        }
        false
    }
}

impl RxChannel<PayloadInfo, NodeContent> for RxRadio {
    type R = RxMetrics;

    fn reset_rx(&mut self) {
        self.in_stats.reset();
        self.in_link_nodes.clear();
        self.rx_metrics.clear();
    }

    fn complete_transfers(
        &mut self,
        mut payloads: Vec<DPayload>,
    ) -> (Vec<DPayload>, Vec<RxMetrics>) {
        payloads.shuffle(&mut self.rng);
        let mut valid = Vec::with_capacity(payloads.len());
        let mut rx_stats_vec = Vec::with_capacity(payloads.len());
        let mut rx_order = 1;

        for payload in payloads.into_iter() {
            self.in_stats.add_attempted(&payload.metadata);
            self.in_link_nodes
                .entry(payload.node_state.node_info.node_class)
                .or_insert(Vec::new())
                .push(payload.node_state.node_info.id);

            let rx_stats = self.measure_rx(rx_order, &payload);
            self.rx_metrics
                .insert(payload.node_state.node_info.id, rx_stats);
            if rx_stats.rx_status == RxStatus::Ok {
                self.in_stats.add_feasible(&payload.metadata);
                valid.push(payload);
            }
            rx_stats_vec.push(rx_stats);
            rx_order += 1;
        }
        (valid, rx_stats_vec)
    }

    fn perform_actions(
        &mut self,
        node_info: &NodeContent,
        payloads: Vec<DPayload>,
    ) -> Vec<DPayload> {
        let mut to_forward = Vec::with_capacity(payloads.len());
        for payload in payloads.into_iter() {
            let payload = self.do_actions(payload, node_info);
            to_forward.push(payload);
        }
        to_forward
    }
}

#[derive(Clone, Debug, TypedBuilder)]
pub struct TxRadio {
    pub rng: Pcg64Mcg,
    #[builder(default)]
    pub actions: HashMap<NodeClass, HashMap<DataType, ActionImpl>>,
    #[builder(default)]
    pub out_link_nodes: HashMap<NodeClass, Vec<NodeId>>,
    #[builder(default)]
    pub out_stats: OutDataStats,
}

impl TxRadio {
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
