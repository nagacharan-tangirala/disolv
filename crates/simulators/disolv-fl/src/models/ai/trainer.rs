use std::cmp::min;
use std::path::PathBuf;

use burn::data::dataloader::batcher::Batcher;
use burn::data::dataset::Dataset;
use burn::module::Module;
use burn::prelude::Backend;
use burn::record::CompactRecorder;
use burn::tensor::backend::AutodiffBackend;
use burn::tensor::ElementConversion;
use log::debug;
use serde::Deserialize;
use typed_builder::TypedBuilder;

use disolv_core::bucket::TimeMS;

use crate::models::ai::mnist::{
    mnist_train, MnistBatcher, MnistInputConfigSettings, MnistModel, MnistTrainingConfig,
};
use crate::models::ai::models::{DatasetType, ModelType};

#[derive(Debug, Clone, Deserialize)]
pub struct TrainerSettings {
    pub(crate) model_type: String,
    pub(crate) no_of_weights: u64,
    pub(crate) mnist_config_settings: Option<MnistInputConfigSettings>,
}

#[derive(Clone, TypedBuilder)]
pub(crate) struct Trainer<B: Backend> {
    pub(crate) model: ModelType<B>,
    pub(crate) no_of_weights: u64,
    pub(crate) output_path: PathBuf,
    pub(crate) config: MnistTrainingConfig,
    #[builder(default)]
    pub(crate) test_data: DatasetType,
    #[builder(default)]
    pub(crate) train_data: DatasetType,
}

impl<B: AutodiffBackend> Trainer<B> {
    pub fn train(&mut self, device: &B::Device) {
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
                    self.output_path.clone(),
                    self.config.clone(),
                    test_dataset.clone(),
                    train_dataset.clone(),
                    mnist.clone(),
                    device.clone(),
                )
            }
            ModelType::Cifar(_) => unimplemented!("cifar is unimplemented"),
        };
        self.train_data.clear();
    }

    pub fn save_model_to_file(&self, step: TimeMS) {
        let mut model_name = step.to_string();
        model_name.push_str("_model");
        match &self.model {
            ModelType::Mnist(mnist) => {
                mnist
                    .clone()
                    .save_file(
                        self.output_path
                            .join("models")
                            .join(model_name)
                            .to_str()
                            .expect("invalid output path"),
                        &CompactRecorder::new(),
                    )
                    .expect("Failed to save model to disk");
            }
            _ => unimplemented!("other models not implemented"),
        }
    }

    pub fn test_model(&self, device: &B::Device) -> f32 {
        match &self.model {
            ModelType::Mnist(mnist) => {
                let test_dataset = match &self.test_data {
                    DatasetType::Mnist(mnist_test) => mnist_test,
                    _ => panic!("Expected mnist test dataset"),
                };
                let total_tests = min(50, (test_dataset.len() as f32 * 0.1) as usize);
                debug!("Validating with {} tests", total_tests);
                mnist_validate(mnist, test_dataset.clone(), total_tests, device.clone())
            }
            _ => unimplemented!("other models not supported"),
        }
    }
}
