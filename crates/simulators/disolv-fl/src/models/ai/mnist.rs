use std::cmp::min;
use std::path::PathBuf;

use burn::config::Config;
use burn::data::dataloader::batcher::Batcher;
use burn::data::dataloader::{DataLoaderBuilder, Dataset};
use burn::data::dataset::vision::{MnistDataset, MnistItem};
use burn::module::{Module, Param};
use burn::nn::conv::{Conv2d, Conv2dConfig};
use burn::nn::loss::CrossEntropyLossConfig;
use burn::nn::{
    BatchNorm, BatchNormConfig, Dropout, DropoutConfig, Gelu, Linear, LinearConfig, PaddingConfig2d,
};
use burn::optim::AdamConfig;
use burn::prelude::{Backend, Tensor, TensorData};
use burn::record::CompactRecorder;
use burn::tensor::backend::AutodiffBackend;
use burn::tensor::{ElementConversion, Int};
use burn::train::metric::{AccuracyMetric, LossMetric};
use burn::train::{ClassificationOutput, LearnerBuilder, TrainOutput, TrainStep, ValidStep};
use log::debug;
use serde::Deserialize;
use typed_builder::TypedBuilder;

use disolv_core::model::{Model, ModelSettings};

use crate::models::data::mnist::{MnistBatch, MnistBatcher, MnistFlDataset};
use crate::simulation::render::CustomRenderer;

#[derive(Clone, Debug, Deserialize)]
pub struct MnistHyperParameters {
    pub num_epochs: Option<usize>,
    pub batch_size: Option<usize>,
    pub num_workers: Option<usize>,
    pub seed: Option<u64>,
    pub learning_rate: Option<f64>,
    pub num_classes: usize,
    pub hidden_size: usize,
    pub drop_out: f64,
}

impl ModelSettings for MnistHyperParameters {}

#[derive(Config, TypedBuilder)]
pub struct MnistTrainingConfig {
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

impl Model for MnistTrainingConfig {
    type Settings = MnistHyperParameters;

    fn with_settings(settings: &Self::Settings) -> Self {
        let optimizer = AdamConfig::new();
        let mut train_config = MnistTrainingConfig::new(optimizer);
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

#[derive(Module, Debug)]
pub struct ConvBlock<B: Backend> {
    pub conv: Conv2d<B>,
    pub norm: BatchNorm<B, 2>,
    pub activation: Gelu,
}

impl<B: Backend> ConvBlock<B> {
    pub fn new(channels: [usize; 2], kernel_size: [usize; 2], device: &B::Device) -> Self {
        let conv = Conv2dConfig::new(channels, kernel_size)
            .with_padding(PaddingConfig2d::Valid)
            .init(device);
        let norm = BatchNormConfig::new(channels[1]).init(device);

        Self {
            conv,
            norm,
            activation: Gelu::new(),
        }
    }

    pub fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 4> {
        let x = self.conv.forward(input);
        let x = self.norm.forward(x);

        self.activation.forward(x)
    }
}

#[derive(Config, Debug)]
pub struct MnistModelConfig {
    num_classes: usize,
    hidden_size: usize,
    #[config(default = "0.5")]
    drop_out: f64,
}

impl MnistModelConfig {
    pub(crate) fn init<B: Backend>(self, device: &B::Device) -> MnistModel<B> {
        MnistModel {
            conv1: ConvBlock::new([1, 8], [3, 3], device),
            conv2: ConvBlock::new([8, 16], [3, 3], device),
            conv3: ConvBlock::new([16, 24], [3, 3], device),
            activation: Gelu::new(),
            linear1: LinearConfig::new(self.hidden_size, 32)
                .with_bias(false)
                .init(device),
            linear2: LinearConfig::new(32, self.num_classes)
                .with_bias(false)
                .init(device),
            dropout: DropoutConfig::new(self.drop_out).init(),
        }
    }
}

#[derive(Module, Debug)]
pub struct MnistModel<B: Backend> {
    pub(crate) conv1: ConvBlock<B>,
    pub(crate) conv2: ConvBlock<B>,
    pub(crate) conv3: ConvBlock<B>,
    pub(crate) dropout: Dropout,
    pub(crate) linear1: Linear<B>,
    pub(crate) linear2: Linear<B>,
    pub(crate) activation: Gelu,
}

impl<B: Backend> MnistModel<B> {
    pub(crate) fn do_fedavg(
        global_model: &mut MnistModel<B>,
        other_models: Vec<MnistModel<B>>,
        device: &B::Device,
    ) {
        let mut linear_weights = other_models
            .iter()
            .map(|model| model.linear1.weight.val())
            .collect();
        let mut avg_linear_tensor = Self::get_average_tensor(linear_weights);
        global_model.linear1.weight = Param::from_data(avg_linear_tensor.into_data(), device);

        linear_weights = other_models
            .iter()
            .map(|model| model.linear2.weight.val())
            .collect();
        avg_linear_tensor = Self::get_average_tensor(linear_weights);
        global_model.linear2.weight = Param::from_data(avg_linear_tensor.into_data(), device);

        let mut conv_weights = other_models
            .iter()
            .map(|model| model.conv1.conv.weight.val())
            .collect();
        let mut avg_conv_tensor = Self::get_average_tensor(conv_weights);
        global_model.conv1.conv.weight = Param::from_data(avg_conv_tensor.into_data(), device);

        conv_weights = other_models
            .iter()
            .map(|model| model.conv2.conv.weight.val())
            .collect();
        avg_conv_tensor = Self::get_average_tensor(conv_weights);
        global_model.conv2.conv.weight = Param::from_data(avg_conv_tensor.into_data(), device);

        conv_weights = other_models
            .iter()
            .map(|model| model.conv3.conv.weight.val())
            .collect();
        avg_conv_tensor = Self::get_average_tensor(conv_weights);
        global_model.conv3.conv.weight = Param::from_data(avg_conv_tensor.into_data(), device);
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

    pub(crate) fn forward(&self, images: Tensor<B, 3>) -> Tensor<B, 2> {
        let [batch_size, height, width] = images.dims();

        let x = images.reshape([batch_size, 1, height, width]);
        let x = self.conv1.forward(x); // [batch_size, 8, _, _]
        let x = self.conv2.forward(x); // [batch_size, 16, _, _]
        let x = self.conv3.forward(x); // [batch_size, 16, _, _]

        let [batch_size, channels, height, width] = x.dims();
        let x = x.reshape([batch_size, channels * height * width]);

        let x = self.dropout.forward(x);
        let x = self.linear1.forward(x);
        let x = self.activation.forward(x);

        self.linear2.forward(x)
    }

    pub(crate) fn forward_classification(&self, item: MnistBatch<B>) -> ClassificationOutput<B> {
        let targets = item.targets;
        let output = self.forward(item.images);
        let loss = CrossEntropyLossConfig::new()
            .init(&output.device())
            .forward(output.clone(), targets.clone());

        ClassificationOutput {
            loss,
            output,
            targets,
        }
    }
}

impl<B: AutodiffBackend> TrainStep<MnistBatch<B>, ClassificationOutput<B>> for MnistModel<B> {
    fn step(&self, batch: MnistBatch<B>) -> TrainOutput<ClassificationOutput<B>> {
        let item = self.forward_classification(batch);
        TrainOutput::new(self, item.loss.backward(), item)
    }
}

impl<B: Backend> ValidStep<MnistBatch<B>, ClassificationOutput<B>> for MnistModel<B> {
    fn step(&self, batch: MnistBatch<B>) -> ClassificationOutput<B> {
        self.forward_classification(batch)
    }
}

pub fn mnist_train<B: AutodiffBackend>(
    output_path: PathBuf,
    config: &MnistTrainingConfig,
    test_data: MnistFlDataset,
    train_data: MnistFlDataset,
    current_model: MnistModel<B>,
    device: B::Device,
) -> MnistModel<B> {
    let batcher_train = MnistBatcher::<B>::new(device.clone());
    let batcher_valid = MnistBatcher::<B::InnerBackend>::new(device.clone());

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

    let updated_model = learner.fit(dataloader_train, dataloader_test).to_owned();
    updated_model
}

pub(crate) fn mnist_validate<B: AutodiffBackend>(
    mnist_model: &MnistModel<B>,
    test_dataset: MnistFlDataset,
    total_tests: usize,
    device: B::Device,
) -> f32 {
    let mut success = 0;

    // TODO: Check if this should be inside the loop
    let batcher = MnistBatcher::new(device);

    for i in 0..total_tests {
        let item = test_dataset.get(i).expect("failed to get item");
        let batch = batcher.batch(vec![item.clone()]);

        let output = mnist_model.forward(batch.images);
        let predicted = output.argmax(1).flatten::<1>(0, 1).into_scalar();
        if predicted.elem::<u8>() == item.label {
            success += 1;
        }
    }
    (success as f32 / total_tests as f32) * 100.0
}
