use std::collections::HashMap;

use burn::tensor::backend::AutodiffBackend;
use log::info;
use serde::Deserialize;

use disolv_core::agent::AgentId;
use disolv_core::model::{Model, ModelSettings};

use crate::models::ai::model::ModelType;

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct AggregationSettings {
    pub(crate) variant: String,
}

impl ModelSettings for AggregationSettings {}

#[derive(Clone)]
pub(crate) enum Aggregator<B: AutodiffBackend> {
    FedAvg(FedAvgAggregator<B>),
}

impl<B: AutodiffBackend> Model for Aggregator<B> {
    type Settings = AggregationSettings;

    fn with_settings(settings: &Self::Settings) -> Self {
        match settings.variant.to_lowercase().as_str() {
            "fedavg" => Aggregator::FedAvg(FedAvgAggregator::new(settings)),
            _ => panic!("Invalid aggregator. Only FedAvg is supported!"),
        }
    }
}

impl<B: AutodiffBackend> Aggregator<B> {
    pub(crate) fn add_local_model(&mut self, local_model: ModelType<B>, agent_id: AgentId) {
        match self {
            Aggregator::FedAvg(aggregator) => aggregator.add_local_model(local_model, agent_id),
        }
    }

    pub(crate) fn aggregate(&mut self, global_model: &mut ModelType<B>, device: &B::Device) {
        match self {
            Aggregator::FedAvg(aggregator) => aggregator.aggregate(global_model, device),
        }
    }

    pub(crate) fn is_model_collected(&self, agent_id: AgentId) -> bool {
        match self {
            Aggregator::FedAvg(aggregator) => aggregator.is_model_collected(agent_id),
        }
    }

    pub(crate) fn has_local_models(&self) -> bool {
        match self {
            Aggregator::FedAvg(aggregator) => !aggregator.local_models.is_empty(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct FedAvgAggregator<B: AutodiffBackend> {
    local_models: HashMap<AgentId, ModelType<B>>,
}

impl<B: AutodiffBackend> FedAvgAggregator<B> {
    fn new(settings: &AggregationSettings) -> Self {
        Self {
            local_models: HashMap::new(),
        }
    }

    fn add_local_model(&mut self, local_model: ModelType<B>, agent_id: AgentId) {
        self.local_models.insert(agent_id, local_model);
    }

    fn is_model_collected(&self, agent_id: AgentId) -> bool {
        self.local_models.contains_key(&agent_id)
    }

    fn aggregate(&mut self, global_model: &mut ModelType<B>, device: &B::Device) {
        info!("{} is the local model count.", self.local_models.len());
        global_model.do_fedavg(self.local_models.clone().into_values().collect(), &device);
        self.local_models.clear();
    }
}
