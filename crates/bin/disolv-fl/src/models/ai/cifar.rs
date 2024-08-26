use burn::nn::conv::Conv2d;
use burn::prelude::Backend;

/// todo: Add cifar10 dataset stuff here.
#[derive(Clone)]
pub struct CifarTrainer<B: Backend> {
    pub(crate) device: B::Device,
    pub(crate) quantity: u64,
    model: CifarModel<B>,
}

#[derive(Clone)]
pub struct CifarModel<B: Backend> {
    pub conv1: Conv2d<B>,
}

#[derive(Clone)]
pub struct CifarFlDataset {}
