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

pub trait Receiver<B, C, D, M, P, Q>
where
    B: Bucket,
    C: ContentType,
    D: DataUnit<C>,
    M: Metadata,
    P: AgentProperties,
    Q: QueryType,
{
    fn receive(&mut self, bucket: &mut B) -> Option<Vec<Payload<C, D, M, P, Q>>>;
    fn receive_sl(&mut self, bucket: &mut B) -> Option<Vec<Payload<C, D, M, P, Q>>>;
}
pub trait FlServer<B, C, D, M, P, Q>
where
    B: Bucket,
    C: ContentType,
    D: DataUnit<C>,
    M: Metadata,
    P: AgentProperties,
    Q: QueryType,
{
    fn process_fl_messages(&mut self, messages: Option<Vec<Payload<C, D, M, P, Q>>>);
    fn update_state(&mut self);
}

/// An enum enclosing different training setups.
#[derive(Clone)]
pub enum ModelType<A: AutodiffBackend, B: Backend> {
    Mnist(MnistTrainer<A>),
    Cifar(CifarTrainer<B>),
}

#[derive(Debug, Clone)]
pub enum DatasetType {
    MNIST(MnistFlDataset),
}