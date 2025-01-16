use burn::prelude::Backend;

use crate::models::ai::cifar::CifarModel;
use crate::models::ai::mnist::MnistModel;

#[derive(Clone)]
pub enum ModelType<B: Backend> {
    Mnist(MnistModel<B>),
    Cifar(CifarModel<B>),
}

impl<B: Backend> ModelType<B> {
    pub fn do_fedavg(&mut self, local_models: Vec<ModelType<B>>, device: &B::Device) {
        match self {
            ModelType::Mnist(mnist_model) => {
                let mnist_models = local_models
                    .into_iter()
                    .map(|model| match model {
                        ModelType::Mnist(mnist) => mnist,
                        _ => panic!("wrong local model sent to aggregate"),
                    })
                    .collect();
                MnistModel::do_fedavg(mnist_model, mnist_models, device)
            }
            ModelType::Cifar(cifar_model) => {
                let cifar_models = local_models
                    .into_iter()
                    .map(|model| match model {
                        ModelType::Cifar(cifar) => cifar,
                        _ => panic!("wrong local model sent to aggregate"),
                    })
                    .collect();
                CifarModel::do_fedavg(cifar_model, cifar_models, device)
            }
            _ => unimplemented!("cifar not implemented"),
        }
    }
}
