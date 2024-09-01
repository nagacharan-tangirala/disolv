use std::path::PathBuf;

use burn::data::dataloader::batcher::Batcher;
use burn::data::dataset::Dataset;
use burn::module::Module;
use burn::prelude::Backend;
use burn::record::CompactRecorder;
use burn::tensor::backend::AutodiffBackend;
use burn::tensor::ElementConversion;
use serde::Deserialize;
use typed_builder::TypedBuilder;

use crate::models::ai::mnist::{
    mnist_train, MnistBatcher, MnistModel, MnistTrainConfigSettings, MnistTrainingConfig,
};
use crate::models::ai::models::{DatasetType, ModelType};

#[derive(Debug, Clone, Deserialize)]
pub struct TrainerSettings {
    pub(crate) model_type: String,
    pub(crate) no_of_weights: u64,
    pub(crate) mnist_config_settings: Option<MnistTrainConfigSettings>,
}

#[derive(Clone, TypedBuilder)]
pub(crate) struct Trainer<B: Backend> {
    pub(crate) model: ModelType<B>,
    pub(crate) device: B::Device,
    pub(crate) no_of_weights: u64,
    pub(crate) output_path: PathBuf,
    pub(crate) config: MnistTrainingConfig,
    #[builder(default)]
    pub(crate) test_data: DatasetType,
    #[builder(default)]
    pub(crate) train_data: DatasetType,
}

impl<B: AutodiffBackend> Trainer<B> {
    pub fn train(&mut self) {
        self.model = match &self.model {
            ModelType::Mnist(mnist) => {
                let test_dataset = match &self.test_data {
                    DatasetType::Mnist(mnist_test) => mnist_test,
                    _ => panic!("Expected mnist test dataset, found something else"),
                };
                let train_dataset = match &self.train_data {
                    DatasetType::Mnist(mnist_test) => mnist_test,
                    _ => panic!("Expected mnist test dataset, found something else"),
                };
                mnist_train(
                    &self.output_path,
                    self.config.clone(),
                    test_dataset.clone(),
                    train_dataset.clone(),
                    mnist.clone(),
                    self.device.clone(),
                )
            }
            ModelType::Cifar(cifar) => unimplemented!("cifar is unimplemented"),
        };
    }

    pub fn save_model_to_file(&self) {
        match &self.model {
            ModelType::Mnist(mnist) => {
                mnist
                    .clone()
                    .save_file(
                        self.output_path
                            .join("model")
                            .to_str()
                            .expect("invalid output path"),
                        &CompactRecorder::new(),
                    )
                    .expect("Failed to save model to disk");
            }
            _ => unimplemented!("other models not implemented"),
        }
    }

    pub fn test_model(&self) -> f32 {
        match &self.model {
            ModelType::Mnist(mnist) => self.mnist_validation(mnist),
            _ => unimplemented!("other models not supported"),
        }
    }

    fn mnist_validation(&self, mnist_model: &MnistModel<B>) -> f32 {
        let test_dataset = match &self.test_data {
            DatasetType::Mnist(mnist_test) => mnist_test,
            _ => panic!("Expected mnist test dataset, found something else"),
        };

        let total_tests = (test_dataset.len() as f32 * 0.1) as usize;
        let mut success = 0;

        for i in 0..total_tests {
            let item = test_dataset.get(i).expect("failed to get item");
            let batcher = MnistBatcher::new(self.device.clone());
            let batch = batcher.batch(vec![item.clone()]);

            let output = mnist_model.forward(batch.images);
            let predicted = output.argmax(1).flatten::<1>(0, 1).into_scalar();
            if predicted.elem::<u8>() == item.label {
                success = success + 1;
            }
        }
        (success as f32 / total_tests as f32) * 100.0
    }
}
