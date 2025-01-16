use std::fmt::{Display, Formatter};

use burn::prelude::Backend;

use crate::models::ai::cifar::CifarModel;
use crate::models::ai::mnist::MnistModel;

/// A trait that represents the training state if the agent is participating in federated
/// learning training process.
#[derive(Default, Copy, Clone, Debug, Eq, PartialEq)]
pub enum ClientState {
    #[default]
    Sensing,
    Informing,
    ReadyToTrain,
    Training,
}

impl Display for ClientState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientState::Sensing => write!(f, "Sensing"),
            ClientState::Informing => write!(f, "Informing"),
            ClientState::ReadyToTrain => write!(f, "ReadyToTrain"),
            ClientState::Training => write!(f, "Training"),
        }
    }
}

#[derive(Clone)]
pub enum ModelType<B: Backend> {
    Mnist(MnistModel<B>),
    Cifar(CifarModel<B>),
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
