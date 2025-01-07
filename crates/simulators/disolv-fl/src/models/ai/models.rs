use std::fmt::{Display, Formatter};

use burn::data::dataset::Dataset;
use burn::data::dataset::vision::MnistItem;
use burn::prelude::Backend;

use crate::models::ai::cifar::{CifarFlDataset, CifarModel};
use crate::models::ai::mnist::{MnistFlDataset, MnistModel};

/// A trait that represents the training state if the agent is participating in federated
/// learning training process.
#[derive(Default, Copy, Clone, Debug, Eq, PartialEq)]
pub enum ClientState {
    #[default]
    Sensing,
    Informing,
    Preparing,
    ReadyToTrain,
    Training,
}

impl Display for ClientState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientState::Sensing => write!(f, "Sensing"),
            ClientState::Preparing => write!(f, "Preparing"),
            ClientState::Informing => write!(f, "Informing"),
            ClientState::ReadyToTrain => write!(f, "ReadyToTrain"),
            ClientState::Training => write!(f, "Training"),
        }
    }
}

/// A trait that represents the server state while carrying a federated learning training session.
#[derive(Copy, Clone, Debug)]
pub enum ServerState {
    Sampling,
    Selecting,
    Broadcasting,
    Waiting,
    Aggregating,
}

#[derive(Clone)]
pub enum ModelType<B: Backend> {
    Mnist(MnistModel<B>),
    Cifar(CifarModel<B>),
}

#[derive(Clone, Default, Debug)]
pub enum DatasetType {
    #[default]
    Empty,
    Mnist(MnistFlDataset),
    Cifar(CifarFlDataset),
}

impl DatasetType {
    pub fn blank(dataset_type: &str) -> Self {
        match dataset_type.to_lowercase().as_str() {
            "mnist" => DatasetType::Mnist(MnistFlDataset::default()),
            _ => unimplemented!("{} datasets are not implemented", dataset_type),
        }
    }

    pub fn data_length(&self) -> usize {
        match self {
            DatasetType::Mnist(mnist) => mnist.len(),
            _ => 0,
        }
    }
    pub fn dataset_type(&self) -> &str {
        match self {
            DatasetType::Mnist(_) => "mnist",
            DatasetType::Cifar(_) => "cifar",
            DatasetType::Empty => "empty",
        }
    }

    pub fn has_data(&self) -> bool {
        match self {
            DatasetType::Mnist(mnist) => !mnist.images.is_empty(),
            DatasetType::Cifar(cifar) => false,
            DatasetType::Empty => false,
        }
    }

    pub fn append_mnist(&mut self, new_item: MnistItem) {
        match self {
            DatasetType::Mnist(mnist) => mnist.images.push(new_item),
            _ => panic!("Trying to push mnist data to wrong dataset"),
        }
    }

    pub fn clear(&mut self) {
        match self {
            DatasetType::Mnist(mnist) => mnist.images.clear(),
            _ => unimplemented!("Unable to clear dataset"),
        }
    }
}

/// A simple enum to define test and train data types.
#[derive(Clone)]
pub enum BatchType {
    Test,
    Train,
}

#[derive(Clone, Copy)]
pub enum ModelDirection {
    NA,
    Sent,
    Received,
}

impl Display for ModelDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelDirection::NA => write!(f, "NA"),
            ModelDirection::Received => write!(f, "Received"),
            ModelDirection::Sent => write!(f, "Sent"),
        }
    }
}

#[derive(Clone, Copy)]
pub enum ModelLevel {
    Local,
    Global,
}

impl Display for ModelLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelLevel::Local => write!(f, "Local"),
            ModelLevel::Global => write!(f, "Global"),
        }
    }
}

#[derive(Clone, Copy)]
pub enum TrainingStatus {
    NA,
    Success,
    Failure,
}

impl Display for TrainingStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TrainingStatus::NA => write!(f, "NA"),
            TrainingStatus::Failure => write!(f, "Failure"),
            TrainingStatus::Success => write!(f, "Success"),
        }
    }
}
