use std::cmp::min;
use std::path::PathBuf;

use burn::data::dataset::Dataset;
use burn::module::Module;
use burn::prelude::Backend;
use burn::record::CompactRecorder;
use burn::tensor::backend::AutodiffBackend;
use log::debug;
use serde::Deserialize;
use typed_builder::TypedBuilder;

use disolv_core::bucket::TimeMS;

use crate::models::ai::cifar::CifarTrainingConfig;
use crate::models::ai::mnist::{
    mnist_train, mnist_validate, MnistHyperParameters, MnistTrainingConfig,
};
use crate::models::ai::model::ModelType;
use crate::models::data::dataset::DatasetType;

#[derive(Debug, Clone, Deserialize)]
pub struct TrainerSettings {
    pub(crate) model_type: String,
    pub(crate) no_of_weights: u64,
    pub(crate) mnist_hyper_parameters: Option<MnistHyperParameters>,
}

#[derive(Clone)]
pub enum TrainingConfig {
    MnistTrain(MnistTrainingConfig),
    CifarTrain(CifarTrainingConfig),
}

#[derive(Clone, TypedBuilder)]
pub(crate) struct Trainer<B: Backend> {
    pub(crate) model: ModelType<B>,
    pub(crate) no_of_weights: u64,
    pub(crate) output_path: PathBuf,
    pub(crate) config: TrainingConfig,
    #[builder(default)]
    pub(crate) test_data: DatasetType,
    #[builder(default)]
    pub(crate) train_data: DatasetType,
}

impl<B: AutodiffBackend> Trainer<B> {
    pub fn train(&mut self, device: &B::Device) {
        self.model = match self.model.clone() {
            ModelType::Mnist(mnist) => {
                let test_dataset = match &self.test_data {
                    DatasetType::Mnist(mnist_test) => mnist_test,
                    _ => panic!("Expected mnist test dataset, found something else"),
                };
                let train_dataset = match &self.train_data {
                    DatasetType::Mnist(mnist_test) => mnist_test,
                    _ => panic!("Expected mnist test dataset, found something else"),
                };
                let train_config = match &self.config {
                    TrainingConfig::MnistTrain(mnist_config) => mnist_config,
                    _ => panic!("Expected mnist training config, found something else"),
                };
                let mnist_model = mnist_train(
                    self.output_path.clone(),
                    train_config,
                    test_dataset.clone(),
                    train_dataset.clone(),
                    mnist.clone(),
                    device.clone(),
                );
                ModelType::Mnist(mnist_model)
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
