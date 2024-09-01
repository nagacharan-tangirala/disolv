use serde::Deserialize;

use disolv_core::agent::AgentId;
use disolv_core::bucket::TimeMS;
use disolv_core::model::{BucketModel, Model, ModelSettings};
use disolv_models::dist::{DistParams, RngSampler};

use crate::models::ai::mnist::MnistFlDataset;
use crate::models::ai::models::{BatchType, DatasetType};

#[derive(Debug, Clone, Deserialize)]
pub struct DistributorSettings {
    pub total_clients: u64,
    pub dataset_type: String,
    pub label_skew: Option<DistParams>,
    pub data_skew: Option<DistParams>,
    pub variant: String,
}

impl ModelSettings for DistributorSettings {}

#[derive(Clone)]
pub(crate) enum DataDistributor {
    Uniform(UniformDistributor),
    NonIid(NonIidDistributor),
}

impl Model for DataDistributor {
    type Settings = DistributorSettings;

    fn with_settings(settings: &Self::Settings) -> Self {
        match settings.variant.to_lowercase().as_str() {
            "uniform" => DataDistributor::Uniform(UniformDistributor::with_settings(settings)),
            _ => unimplemented!("other distributions to be implemented"),
        }
    }
}

impl BucketModel for DataDistributor {
    fn init(&mut self, step: TimeMS) {
        match self {
            DataDistributor::Uniform(uniform) => uniform.init(step),
            DataDistributor::NonIid(non_iid) => non_iid.init(step),
        }
    }

    fn stream_data(&mut self, step: TimeMS) {
        match self {
            DataDistributor::Uniform(uniform) => uniform.stream_data(step),
            DataDistributor::NonIid(non_iid) => non_iid.stream_data(step),
        }
    }

    fn before_agent_step(&mut self, step: TimeMS) {
        match self {
            DataDistributor::Uniform(uniform) => uniform.before_agent_step(step),
            DataDistributor::NonIid(non_iid) => non_iid.before_agent_step(step),
        }
    }
}

impl DataDistributor {
    pub fn training_data(&mut self, agent_id: AgentId) -> Option<DatasetType> {
        match self {
            DataDistributor::Uniform(uniform) => uniform.training_data(agent_id),
            DataDistributor::NonIid(non_iid) => non_iid.training_data(agent_id),
        }
    }

    pub fn test_data(&mut self, agent_id: AgentId) -> Option<DatasetType> {
        match self {
            DataDistributor::Uniform(uniform) => uniform.test_data(agent_id),
            DataDistributor::NonIid(non_iid) => non_iid.test_data(agent_id),
        }
    }
}

#[derive(Clone)]
pub struct UniformDistributor {
    pub total_clients: u64,
    pub dataset_type: String,
    pub test_data: Vec<DatasetType>,
    pub train_data: Vec<DatasetType>,
}

impl Model for UniformDistributor {
    type Settings = DistributorSettings;

    fn with_settings(settings: &Self::Settings) -> Self {
        Self {
            total_clients: settings.total_clients,
            dataset_type: settings.dataset_type.to_lowercase(),
            train_data: Vec::with_capacity(settings.total_clients as usize),
            test_data: Vec::with_capacity(settings.total_clients as usize),
        }
    }
}

impl BucketModel for UniformDistributor {
    fn init(&mut self, _step: TimeMS) {
        match self.dataset_type.to_lowercase().as_str() {
            "mnist" => {
                let train_data = MnistFlDataset::new(BatchType::Train);
                let partition_size = train_data.images.len() / self.total_clients as usize;
                train_data
                    .images
                    .chunks(partition_size)
                    .for_each(|image_chunk| {
                        self.train_data
                            .push(DatasetType::Mnist(MnistFlDataset::with_images(
                                image_chunk.to_vec(),
                            )))
                    });
                let test_data = MnistFlDataset::new(BatchType::Test);
                test_data
                    .images
                    .chunks(partition_size)
                    .for_each(|image_chunk| {
                        self.test_data
                            .push(DatasetType::Mnist(MnistFlDataset::with_images(
                                image_chunk.to_vec(),
                            )))
                    });
            }
            _ => panic!("Invalid dataset type"),
        }
    }

    fn stream_data(&mut self, step: TimeMS) {}

    fn before_agent_step(&mut self, step: TimeMS) {}
}

impl UniformDistributor {
    fn training_data(&mut self, _agent_id: AgentId) -> Option<DatasetType> {
        self.train_data.pop()
    }

    fn test_data(&mut self, _agent_id: AgentId) -> Option<DatasetType> {
        self.test_data.pop()
    }
}

#[derive(Clone)]
pub(crate) struct NonIidDistributor {
    pub total_clients: u64,
    pub label_skew: Option<RngSampler>,
    pub data_skew: Option<RngSampler>,
    pub dataset_type: String,
    pub test_data: Vec<DatasetType>,
    pub train_data: Vec<DatasetType>,
}

impl Model for NonIidDistributor {
    type Settings = DistributorSettings;

    fn with_settings(settings: &Self::Settings) -> Self {
        let label_skew = match settings.label_skew.clone() {
            Some(val) => Some(RngSampler::new(val)),
            None => None,
        };
        let data_skew = match settings.data_skew.clone() {
            Some(val) => Some(RngSampler::new(val)),
            None => None,
        };
        let mut train_data = DatasetType::Empty;
        let mut test_data = DatasetType::Empty;
        match settings.dataset_type.to_lowercase().as_str() {
            "mnist" => {
                train_data = DatasetType::Mnist(MnistFlDataset::new(BatchType::Train));
                test_data = DatasetType::Mnist(MnistFlDataset::new(BatchType::Test));
            }
            _ => panic!("Invalid dataset type"),
        }
        Self {
            total_clients: settings.total_clients,
            data_skew,
            label_skew,
            dataset_type: settings.dataset_type.to_lowercase(),
            train_data: Vec::with_capacity(settings.total_clients as usize),
            test_data: Vec::with_capacity(settings.total_clients as usize),
        }
    }
}

impl BucketModel for NonIidDistributor {
    fn init(&mut self, _step: TimeMS) {
        match self.dataset_type.to_lowercase().as_str() {
            "mnist" => {
                let train_data = MnistFlDataset::new(BatchType::Train);
                let partition_size = train_data.images.len() / self.total_clients as usize;
                // todo: partition the data according to the skews.
            }
            _ => panic!("Invalid dataset type"),
        }
    }

    fn stream_data(&mut self, step: TimeMS) {
        todo!()
    }

    fn before_agent_step(&mut self, step: TimeMS) {
        todo!()
    }
}

impl NonIidDistributor {
    fn training_data(&mut self, _agent_id: AgentId) -> Option<DatasetType> {
        self.train_data.pop()
    }

    fn test_data(&mut self, _agent_id: AgentId) -> Option<DatasetType> {
        self.test_data.pop()
    }
}
