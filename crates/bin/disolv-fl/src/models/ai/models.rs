use std::fmt::Display;

use burn::data::dataset::vision::MnistItem;
use burn::prelude::Backend;

use disolv_core::agent::Agent;
use disolv_core::bucket::Bucket;

use crate::models::ai::cifar::{CifarFlDataset, CifarModel};
use crate::models::ai::mnist::{MnistFlDataset, MnistModel};

/// A trait that represents the training state if the agent is participating in federated
/// learning training process.
#[derive(Default, Copy, Clone, Debug)]
pub enum ClientState {
    #[default]
    Sensing,
    Training,
}

impl ClientState {
    pub const fn value(&self) -> u64 {
        match self {
            ClientState::Sensing => 10,
            ClientState::Training => 20,
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

#[derive(Clone, Default)]
pub enum DatasetType {
    #[default]
    Empty,
    Mnist(MnistFlDataset),
    Cifar(CifarFlDataset),
}

impl DatasetType {
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
            _ => panic!("Trying to append wrong item to mnist data"),
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

pub trait FlAgent<B, C, D, M, P, Q>
where
    B: Bucket,
    C: ContentType,
    D: DataUnit<C>,
    M: Metadata,
    P: AgentProperties,
    Q: QueryType,
{
    fn handle_fl_messages(
        &mut self,
        bucket: &mut B,
        messages: &mut Option<Vec<Payload<C, D, M, P, Q>>>,
    );
    fn update_state(&mut self);
}
