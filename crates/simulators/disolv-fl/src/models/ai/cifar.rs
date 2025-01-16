use std::path::PathBuf;

use burn::config::Config;
use burn::data::dataloader::DataLoaderBuilder;
use burn::module::{Module, Param};
use burn::nn::{Dropout, DropoutConfig, Gelu, Linear, LinearConfig, PaddingConfig2d, Relu};
use burn::nn::conv::{Conv2d, Conv2dConfig};
use burn::nn::loss::CrossEntropyLossConfig;
use burn::nn::pool::{MaxPool2d, MaxPool2dConfig};
use burn::optim::AdamConfig;
use burn::prelude::Backend;
use burn::record::CompactRecorder;
use burn::tensor::{Int, Tensor};
use burn::tensor::backend::AutodiffBackend;
use burn::train::{ClassificationOutput, LearnerBuilder, TrainOutput, TrainStep, ValidStep};
use burn::train::metric::{AccuracyMetric, LossMetric};
use log::debug;
use serde::Deserialize;
use typed_builder::TypedBuilder;

use disolv_core::model::{Model, ModelSettings};

use crate::models::ai::mnist::{MnistModel, MnistTrainingConfig};
use crate::models::data::cifar::CifarFlDataset;
use crate::models::data::dataset::SampleBatcher;
use crate::models::data::mnist::{MnistBatch, MnistFlDataset};
use crate::simulation::render::CustomRenderer;

#[derive(Clone, Debug, Deserialize)]
pub struct CifarHyperParameters {
    pub num_epochs: Option<usize>,
    pub batch_size: Option<usize>,
    pub num_workers: Option<usize>,
    pub seed: Option<u64>,
    pub learning_rate: Option<f64>,
    pub num_classes: usize,
    pub hidden_size: usize,
    pub drop_out: f64,
}

impl ModelSettings for CifarHyperParameters {}

#[derive(Config, TypedBuilder)]
pub struct CifarTrainingConfig {
    pub optimizer: AdamConfig,
    #[config(default = 1)]
    pub num_epochs: usize,
    #[config(default = 100)]
    pub batch_size: usize,
    #[config(default = 3)]
    pub num_workers: usize,
    #[config(default = 42)]
    pub seed: u64,
    #[config(default = 1.0e-4)]
    pub learning_rate: f64,
}

impl Model for CifarTrainingConfig {
    type Settings = CifarHyperParameters;

    fn with_settings(settings: &Self::Settings) -> Self {
        let optimizer = AdamConfig::new();
        let mut train_config = CifarTrainingConfig::new(optimizer);
        if let Some(num_epochs) = settings.num_epochs {
            train_config.num_epochs = num_epochs
        }
        if let Some(batch_size) = settings.batch_size {
            train_config.batch_size = batch_size;
        }
        if let Some(num_workers) = settings.num_workers {
            train_config.num_workers = num_workers;
        }
        if let Some(seed) = settings.seed {
            train_config.seed = seed;
        }
        if let Some(learning_rate) = settings.learning_rate {
            train_config.learning_rate = learning_rate;
        }
        train_config
    }
}
#[derive(Clone)]
pub struct CifarTrainer<B: Backend> {
    pub(crate) device: B::Device,
    pub(crate) quantity: u64,
    model: CifarModel<B>,
}

#[derive(Config, Debug)]
pub struct CifarModelConfig {
    num_classes: usize,
    #[config(default = "0.5")]
    drop_out: f64,
}

impl CifarModelConfig {
    pub(crate) fn init<B: Backend>(self, device: &B::Device) -> CifarModel<B> {
        let conv1 = Conv2dConfig::new([3, 32], [3, 3])
            .with_padding(PaddingConfig2d::Same)
            .init(device);
        let conv2 = Conv2dConfig::new([32, 32], [3, 3])
            .with_padding(PaddingConfig2d::Same)
            .init(device);

        let conv3 = Conv2dConfig::new([32, 64], [3, 3])
            .with_padding(PaddingConfig2d::Same)
            .init(device);
        let conv4 = Conv2dConfig::new([64, 64], [3, 3])
            .with_padding(PaddingConfig2d::Same)
            .init(device);

        let conv5 = Conv2dConfig::new([64, 128], [3, 3])
            .with_padding(PaddingConfig2d::Same)
            .init(device);
        let conv6 = Conv2dConfig::new([128, 128], [3, 3])
            .with_padding(PaddingConfig2d::Same)
            .init(device);

        let pool = MaxPool2dConfig::new([2, 2]).with_strides([2, 2]).init();

        let fc1 = LinearConfig::new(2048, 128).init(device);
        let fc2 = LinearConfig::new(128, self.num_classes).init(device);

        let dropout = DropoutConfig::new(0.3).init();

        CifarModel {
            activation: Relu::new(),
            dropout,
            pool,
            conv1,
            conv2,
            conv3,
            conv4,
            conv5,
            conv6,
            fc1,
            fc2,
        }
    }
}

#[derive(Module, Debug)]
pub struct CifarModel<B: Backend> {
    activation: Relu,
    dropout: Dropout,
    pool: MaxPool2d,
    conv1: Conv2d<B>,
    conv2: Conv2d<B>,
    conv3: Conv2d<B>,
    conv4: Conv2d<B>,
    conv5: Conv2d<B>,
    conv6: Conv2d<B>,
    fc1: Linear<B>,
    fc2: Linear<B>,
}

impl<B: Backend> CifarModel<B> {
    pub(crate) fn do_fedavg(
        global_model: &mut CifarModel<B>,
        other_models: Vec<CifarModel<B>>,
        device: &B::Device,
    ) {
        let mut linear_weights = other_models
            .iter()
            .map(|model| model.fc1.weight.val())
            .collect();
        let mut avg_linear_tensor = Self::get_average_tensor(linear_weights);
        global_model.fc1.weight = Param::from_data(avg_linear_tensor.into_data(), device);

        linear_weights = other_models
            .iter()
            .map(|model| model.fc2.weight.val())
            .collect();
        avg_linear_tensor = Self::get_average_tensor(linear_weights);
        global_model.fc2.weight = Param::from_data(avg_linear_tensor.into_data(), device);

        let mut conv_weights = other_models
            .iter()
            .map(|model| model.conv1.weight.val())
            .collect();
        let mut avg_conv_tensor = Self::get_average_tensor(conv_weights);
        global_model.conv1.weight = Param::from_data(avg_conv_tensor.into_data(), device);

        conv_weights = other_models
            .iter()
            .map(|model| model.conv2.weight.val())
            .collect();
        avg_conv_tensor = Self::get_average_tensor(conv_weights);
        global_model.conv2.weight = Param::from_data(avg_conv_tensor.into_data(), device);

        conv_weights = other_models
            .iter()
            .map(|model| model.conv3.weight.val())
            .collect();
        avg_conv_tensor = Self::get_average_tensor(conv_weights);
        global_model.conv3.weight = Param::from_data(avg_conv_tensor.into_data(), device);

        conv_weights = other_models
            .iter()
            .map(|model| model.conv4.weight.val())
            .collect();
        avg_conv_tensor = Self::get_average_tensor(conv_weights);
        global_model.conv4.weight = Param::from_data(avg_conv_tensor.into_data(), device);

        conv_weights = other_models
            .iter()
            .map(|model| model.conv5.weight.val())
            .collect();
        avg_conv_tensor = Self::get_average_tensor(conv_weights);
        global_model.conv5.weight = Param::from_data(avg_conv_tensor.into_data(), device);

        conv_weights = other_models
            .iter()
            .map(|model| model.conv6.weight.val())
            .collect();
        avg_conv_tensor = Self::get_average_tensor(conv_weights);
        global_model.conv6.weight = Param::from_data(avg_conv_tensor.into_data(), device);
    }

    fn get_average_tensor<const D: usize>(weights: Vec<Tensor<B, D>>) -> Tensor<B, D> {
        let mut avg_tensor = weights.first().expect("empty weights not possible").clone();
        let total_weights = weights.len() as f32;
        weights
            .into_iter()
            .skip(1)
            .for_each(|tensor| avg_tensor = avg_tensor.clone().add(tensor));
        avg_tensor.div_scalar(total_weights)
    }

    pub fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 2> {
        let x = self.conv1.forward(input);
        let x = self.activation.forward(x);
        let x = self.conv2.forward(x);
        let x = self.activation.forward(x);
        let x = self.pool.forward(x);
        let x = self.dropout.forward(x);

        let x = self.conv3.forward(x);
        let x = self.activation.forward(x);
        let x = self.conv4.forward(x);
        let x = self.activation.forward(x);
        let x = self.pool.forward(x);
        let x = self.dropout.forward(x);

        let x = self.conv5.forward(x);
        let x = self.activation.forward(x);
        let x = self.conv6.forward(x);
        let x = self.activation.forward(x);
        let x = self.pool.forward(x);
        let x = self.dropout.forward(x);

        let x = x.flatten(1, 3);

        let x = self.fc1.forward(x);
        let x = self.activation.forward(x);
        let x = self.dropout.forward(x);

        self.fc2.forward(x)
    }

    pub fn forward_classification(
        &self,
        images: Tensor<B, 4>,
        targets: Tensor<B, 1, Int>,
    ) -> ClassificationOutput<B> {
        let output = self.forward(images);
        let loss = CrossEntropyLossConfig::new()
            .init(&output.device())
            .forward(output.clone(), targets.clone());

        ClassificationOutput::new(loss, output, targets)
    }
}

impl<B: AutodiffBackend> TrainStep<ClassificationBatch<B>, ClassificationOutput<B>>
    for CifarModel<B>
{
    fn step(&self, batch: ClassificationBatch<B>) -> TrainOutput<ClassificationOutput<B>> {
        let item = self.forward_classification(batch.images, batch.targets);
        TrainOutput::new(self, item.loss.backward(), item)
    }
}

impl<B: Backend> ValidStep<ClassificationBatch<B>, ClassificationOutput<B>> for CifarModel<B> {
    fn step(&self, batch: ClassificationBatch<B>) -> ClassificationOutput<B> {
        self.forward_classification(batch.images, batch.targets)
    }
}

pub fn cifar_train<B: AutodiffBackend>(
    output_path: PathBuf,
    config: CifarTrainingConfig,
    test_data: CifarFlDataset,
    train_data: CifarFlDataset,
    current_model: CifarModel<B>,
    device: B::Device,
) -> CifarModel<B> {
    let batcher_train = SampleBatcher::<B>::new(device.clone());
    let batcher_valid = SampleBatcher::<B::InnerBackend>::new(device.clone());

    let dataloader_train = DataLoaderBuilder::new(batcher_train)
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(train_data);

    let dataloader_test = DataLoaderBuilder::new(batcher_valid)
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(test_data);

    debug!("Training dataset size is {}", dataloader_train.num_items());

    let learner = LearnerBuilder::new(output_path.to_str().expect("invalid output path"))
        .metric_train_numeric(AccuracyMetric::new())
        .metric_valid_numeric(AccuracyMetric::new())
        .metric_train_numeric(LossMetric::new())
        .metric_valid_numeric(LossMetric::new())
        .with_file_checkpointer(CompactRecorder::new())
        .devices(vec![device])
        .num_epochs(config.num_epochs)
        .renderer(CustomRenderer {})
        .build(current_model, config.optimizer.init(), config.learning_rate);

    learner.fit(dataloader_train, dataloader_test)
}
