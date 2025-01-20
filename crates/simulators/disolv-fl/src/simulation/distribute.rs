use log::debug;
use serde::Deserialize;

use disolv_core::agent::AgentId;
use disolv_core::bucket::TimeMS;
use disolv_core::model::{BucketModel, Model, ModelSettings};
use disolv_models::dist::{DistParams, SeriesSampler};

use crate::models::ai::common::BatchType;
use crate::models::data::cifar::CifarFlDataset;
use crate::models::data::dataset::DatasetType;
use crate::models::data::mnist::MnistFlDataset;

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
            "noniid" => DataDistributor::NonIid(NonIidDistributor::with_settings(settings)),
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

    pub fn server_test_data(&mut self) -> DatasetType {
        match self {
            DataDistributor::Uniform(uniform) => uniform.server_test_data(),
            DataDistributor::NonIid(non_iid) => non_iid.server_test_data(),
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
        debug!("Total clients are {}", self.total_clients);
        match self.dataset_type.to_lowercase().as_str() {
            "mnist" => {
                let train_data = MnistFlDataset::new(BatchType::Train);
                let mut partition_size = train_data.images.len() / self.total_clients as usize;
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
                partition_size = test_data.images.len() / self.total_clients as usize;
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
            "cifar" => {
                let train_data = CifarFlDataset::new(BatchType::Train);
                let mut partition_size = train_data.images.len() / self.total_clients as usize;
                train_data
                    .images
                    .chunks(partition_size)
                    .for_each(|image_chunk| {
                        self.train_data
                            .push(DatasetType::Cifar(CifarFlDataset::with_images(
                                image_chunk.to_vec(),
                            )))
                    });
                let test_data = CifarFlDataset::new(BatchType::Test);
                partition_size = test_data.images.len() / self.total_clients as usize;
                test_data
                    .images
                    .chunks(partition_size)
                    .for_each(|image_chunk| {
                        self.test_data
                            .push(DatasetType::Cifar(CifarFlDataset::with_images(
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

    fn server_test_data(&mut self) -> DatasetType {
        match self.dataset_type.to_lowercase().as_str() {
            "mnist" => DatasetType::Mnist(MnistFlDataset::new(BatchType::Test)),
            "cifar" => DatasetType::Cifar(CifarFlDataset::new(BatchType::Test)),
            _ => panic!("Invalid dataset type"),
        }
    }
}

#[derive(Clone)]
pub(crate) struct NonIidDistributor {
    pub total_clients: u64,
    pub label_skew: Option<SeriesSampler>,
    pub data_skew: Option<SeriesSampler>,
    pub dataset_type: String,
    pub test_data: Vec<DatasetType>,
    pub train_data: Vec<DatasetType>,
}

impl Model for NonIidDistributor {
    type Settings = DistributorSettings;

    fn with_settings(settings: &Self::Settings) -> Self {
        let label_skew = settings.label_skew.clone().map(SeriesSampler::new);
        let data_skew = settings.data_skew.clone().map(SeriesSampler::new);
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
        self.apply_data_skew();
    }

    fn stream_data(&mut self, step: TimeMS) {}

    fn before_agent_step(&mut self, step: TimeMS) {}
}

impl NonIidDistributor {
    fn training_data(&mut self, _agent_id: AgentId) -> Option<DatasetType> {
        self.train_data.pop()
    }

    fn test_data(&mut self, _agent_id: AgentId) -> Option<DatasetType> {
        self.test_data.pop()
    }

    fn server_test_data(&mut self) -> DatasetType {
        match self.dataset_type.to_lowercase().as_str() {
            "mnist" => DatasetType::Mnist(MnistFlDataset::new(BatchType::Test)),
            "cifar" => DatasetType::Cifar(CifarFlDataset::new(BatchType::Test)),
            _ => panic!("Invalid dataset type"),
        }
    }

    fn apply_data_skew(&mut self) {
        if let Some(data_skew) = &mut self.data_skew {
            let ratios = data_skew.sample();
            if ratios.len() != self.total_clients as usize {
                panic!("total clients should be same as data skew distribution parameters");
            }
            match self.dataset_type.to_lowercase().as_str() {
                "mnist" => {
                    let mnist_chunks =
                        MnistFlDataset::split_with_ratios(ratios.clone(), BatchType::Train);
                    mnist_chunks
                        .into_iter()
                        .for_each(|dataset| self.train_data.push(DatasetType::Mnist(dataset)));

                    let mnist_chunks = MnistFlDataset::split_with_ratios(ratios, BatchType::Test);
                    mnist_chunks
                        .into_iter()
                        .for_each(|dataset| self.test_data.push(DatasetType::Mnist(dataset)));
                }
                "cifar" => {
                    let cifar_chunks =
                        CifarFlDataset::split_with_ratios(ratios.clone(), BatchType::Train);
                    cifar_chunks
                        .into_iter()
                        .for_each(|dataset| self.train_data.push(DatasetType::Cifar(dataset)));

                    let mnist_chunks = CifarFlDataset::split_with_ratios(ratios, BatchType::Test);
                    mnist_chunks
                        .into_iter()
                        .for_each(|dataset| self.test_data.push(DatasetType::Cifar(dataset)));
                }
                _ => panic!("Invalid dataset type"),
            }
        }
    }
}
