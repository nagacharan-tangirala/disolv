use burn::prelude::Backend;
use burn::tensor::backend::AutodiffBackend;

use disolv_core::agent::{Agent, AgentProperties};
use disolv_core::bucket::Bucket;
use disolv_core::message::{ContentType, DataUnit, Metadata, Payload, QueryType};

use crate::models::ai::cifar::CifarTrainer;
use crate::models::ai::mnist::{MnistFlDataset, MnistTrainer};

/// A trait that represents the training state if the agent is participating in federated
/// learning training process.
#[derive(Default, Copy, Clone, Debug)]
pub enum ClientState {
    #[default]
    Sensing,
    Ongoing,
    Complete,
    Fail,
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
pub enum ModelType<A: AutodiffBackend, B: Backend> {
    Mnist(MnistModel<B>),
    Cifar(CifarModel<A>),
}

/// An enum enclosing different training setups.
#[derive(Clone)]
pub enum TrainerType<A: AutodiffBackend, B: Backend> {
    Mnist(MnistTrainer<A>),
    Cifar(CifarTrainer<B>),
}

impl<A: AutodiffBackend, B: Backend> TrainerType<A, B> {
    pub(crate) fn device(&self) -> &B::Device {
        match self {
            TrainerType::Mnist(mnist) => mnist.device.clone(),
            TrainerType::Cifar(cifar) => cifar.device.clone(),
        }
    }

    pub(crate) fn no_of_weights(&self) -> u64 {
        match self {
            TrainerType::Mnist(mnist) => mnist.quantity,
            TrainerType::Cifar(cifar) => cifar.quantity,
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
