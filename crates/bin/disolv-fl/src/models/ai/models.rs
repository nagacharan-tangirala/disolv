use std::fmt::Display;

use burn::data::dataset::vision::MnistItem;
use burn::prelude::Backend;

use disolv_core::agent::Agent;
use disolv_core::bucket::Bucket;

use crate::models::ai::cifar::{CifarFlDataset, CifarModel};
use crate::models::ai::mnist::{MnistFlDataset, MnistModel};

/// A trait that represents the training state if the agent is participating in federated
/// learning training process.
#[derive(Default, Copy, Clone, Debug, Eq, PartialEq)]
pub enum ClientState {
    #[default]
    Sensing,
    Waiting,
    Training,
}

impl ClientState {
    pub const fn value(&self) -> u64 {
        match self {
            ClientState::Sensing => 10,
            ClientState::Waiting => 20,
            ClientState::Training => 30,
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
            _ => unimplemented!("other datasets are not implemented"),
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
            DatasetType::Mnist(mnist) => mnist.images.len() > 0,
            DatasetType::Cifar(cifar) => false,
            DatasetType::Empty => false,
        }
    }

    pub fn length(&self) -> usize {
        match self {
            DatasetType::Mnist(mnist) => mnist.images.len(),
            _ => 0,
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

pub trait FlClient<B>: Agent<B>
where
    B: Bucket,
{
    fn training_state(&self) -> ClientState;
}
