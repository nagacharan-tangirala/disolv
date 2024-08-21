use burn::prelude::Backend;
use burn::tensor::backend::AutodiffBackend;
use serde::Deserialize;

use crate::models::ai::mnist::MnistModel;
use crate::models::ai::models::ModelType;

#[derive(Clone, Deserialize)]
pub(crate) struct AggregationSettings {
    pub(crate) c: f64,
    pub(crate) sample_size: f64,
    pub(crate) variant: String,
}

#[derive(Debug, Clone)]
pub(crate) enum Aggregator<A: AutodiffBackend, B: Backend> {
    FedAvg(FedAvgAggregator<A, B>),
}

impl<A: AutodiffBackend, B: Backend> Aggregator<A, B> {
    pub(crate) fn new(settings: &AggregationSettings) -> Self {
        match settings.variant.to_lowercase().as_str() {
            "fedavg" => Aggregator::FedAvg(FedAvgAggregator::new(settings)),
            _ => panic!("Invalid aggregator. Only FedAvg is supported!"),
        }
    }

    pub(crate) fn add_local_model(&mut self, local_model: ModelType<A, B>) {
        match self {
            Aggregator::FedAvg(aggregator) => aggregator.add_local_model(local_model),
        }
    }

    pub(crate) fn aggregate(
        &self,
        global_model: ModelType<A, B>,
        device: &B::Device,
    ) -> ModelType<A, B> {
        match self {
            Aggregator::FedAvg(aggregator) => aggregator.aggregate(global_model, device),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FedAvgAggregator<A: AutodiffBackend, B: Backend> {
    local_models: Vec<ModelType<A, B>>,
}

impl<A: AutodiffBackend, B: Backend> FedAvgAggregator<A, B> {
    fn new(settings: &AggregationSettings) -> Self {
        Self {
            local_models: Vec::new(),
        }
    }

    pub(crate) fn add_local_model(&mut self, local_model: ModelType<A, B>) {
        self.local_models.push(local_model)
    }

    pub(crate) fn aggregate(
        &self,
        global_model: ModelType<A, B>,
        device: &B::Device,
    ) -> ModelType<A, B> {
        match global_model {
            ModelType::Mnist(mnist_model) => {
                let local_models = self
                    .local_models
                    .iter()
                    .map(|model| match model {
                        ModelType::Mnist(mnist) => mnist,
                        _ => panic!("wrong local model sent to aggregate"),
                    })
                    .collect();
                let new_global_model = MnistModel::do_fedavg(mnist_model, local_models, device);
                ModelType::Mnist(new_global_model)
            }
            _ => unimplemented!("cifar not implemented"),
        }
    }
}
