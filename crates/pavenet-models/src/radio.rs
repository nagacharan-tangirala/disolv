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
            actions.entry(action.data_type).or_insert(rule);
        }
        self.actions = actions;
    }

    fn measure_sl_rx(&mut self, payloads: &Vec<DPayload>) {
        let mut rx_order = 1;
        for payload in payloads.iter() {
            let mut sl_stats = RxMetrics::new(payload, rx_order);
            match self
                .models
                .latency_type
                .measure(&sl_stats, &payload.metadata)
            {
                Feasibility::Feasible(latency) => sl_stats.latency = latency,
                Feasibility::Infeasible(latency) => {
                    sl_stats.latency = latency;
                    sl_stats.rx_status = RxStatus::Fail;
                    sl_stats.rx_fail_reason = RxFailReason::LatencyLimit;
                    self.sl_metrics.push(sl_stats);
                    rx_order += 1;
                    continue;
                }
            };
            sl_stats.rx_status = RxStatus::Ok;
            self.sl_metrics.push(sl_stats);
            rx_order += 1;
        }
    }

    pub fn sl_metrics(&mut self) -> Vec<RxMetrics> {
        self.sl_metrics.clone()
    }
}

impl SlChannel<PayloadInfo, NodeContent> for SlRadio {
    fn reset(&mut self) {
        self.in_stats.reset();
        self.in_link_nodes.clear();
        self.sl_metrics.clear();
        self.sl_payloads.clear();
    }

    fn prepare_transfer(&mut self, mut payload: DPayload) -> DPayload {
        self.out_stats.update(&payload.metadata);
        self.out_link_nodes.push(payload.node_state.node_info.id);

        payload.metadata.data_blobs.iter_mut().for_each(|blob| {
            let new_action = match self.actions.get(&blob.data_type) {
                Some(action) => action,
                None => {
                    error!("No action found for data type {}", blob.data_type);
                    panic!("Action missing for data type {}", blob.data_type);
                }
            };
            assign_actions(blob, new_action);
        });
        payload
    }

    fn do_receive(&mut self, node_info: &NodeContent, mut payloads: Vec<DPayload>) {
        payloads.shuffle(&mut self.rng);
        self.measure_sl_rx(&payloads);

        payloads
            .into_iter()
            .zip(self.sl_metrics.iter())
            .for_each(|(payload, rx_stat)| {
                self.in_stats.add_attempted(&payload.metadata);
                self.in_link_nodes.push(payload.node_state.node_info.id);

                if rx_stat.rx_status == RxStatus::Ok {
                    self.in_stats.add_feasible(&payload.metadata);
                    self.sl_payloads.push(payload.clone());
                }
            });

        self.sl_payloads
            .iter_mut()
            .for_each(|payload| do_actions(payload, node_info));
    }
}
