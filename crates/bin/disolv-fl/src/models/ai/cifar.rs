use burn::prelude::Backend;

use crate::models::ai::mnist::MnistModel;

/// todo: Add cifar10 dataset stuff here.
#[derive(Clone)]
pub struct CifarTrainer<B: Backend> {
    model: MnistModel<B>,
}
