use std::fmt::Display;
use std::path::PathBuf;

use burn::backend::wgpu::WgpuDevice;
use burn::config::Config;
use burn::data::dataloader::{DataLoaderBuilder, Dataset};
use burn::data::dataloader::batcher::Batcher;
use burn::data::dataset::vision::{MnistDataset, MnistItem};
use burn::module::{Module, Param};
use burn::nn::{Dropout, DropoutConfig, Linear, LinearConfig, Relu};
use burn::nn::conv::{Conv2d, Conv2dConfig};
use burn::nn::loss::CrossEntropyLoss;
use burn::nn::pool::{AdaptiveAvgPool2d, AdaptiveAvgPool2dConfig};
use burn::optim::AdamConfig;
use burn::prelude::{Backend, Tensor, TensorData};
use burn::record::CompactRecorder;
use burn::tensor::{ElementConversion, Int};
use burn::tensor::backend::AutodiffBackend;
use burn::train::{ClassificationOutput, LearnerBuilder, TrainOutput, TrainStep, ValidStep};
use burn::train::metric::{AccuracyMetric, LossMetric};
use serde::Deserialize;
use typed_builder::TypedBuilder;

use disolv_core::model::{Model, ModelSettings};

use crate::models::ai::models::{BatchType, ModelType};
use crate::simulation::render::CustomRenderer;

#[derive(Default, Clone, Debug)]
pub struct MnistFlDataset {
    pub images: Vec<MnistItem>,
}

impl MnistFlDataset {
    pub fn new(dataset_type: BatchType) -> Self {
        let images = match dataset_type {
            BatchType::Test => MnistDataset::test().iter().collect(),
            BatchType::Train => MnistDataset::train().iter().collect(),
        };
        Self { images }
    }

    pub fn with_images(images: Vec<MnistItem>) -> Self {
        Self { images }
    }
}

impl Dataset<MnistItem> for MnistFlDataset {
    fn get(&self, index: usize) -> Option<MnistItem> {
        self.images.get(index).cloned()
    }

    fn len(&self) -> usize {
        self.images.len()
    }
}

#[derive(Clone, TypedBuilder)]
pub struct MnistBatcher<B: Backend> {
    pub device: B::Device,
}

impl<B: Backend> MnistBatcher<B> {
    pub fn new(device: B::Device) -> Self {
        Self { device }
    }
}

#[derive(Clone, Debug)]
pub struct MnistBatch<B: Backend> {
    pub images: Tensor<B, 3>,
    pub targets: Tensor<B, 1, Int>,
}

impl<B: Backend> Batcher<MnistItem, MnistBatch<B>> for MnistBatcher<B> {
    fn batch(&self, items: Vec<MnistItem>) -> MnistBatch<B> {
        let images = items
            .iter()
            .map(|item| TensorData::from(item.image).convert::<B::FloatElem>())
            .map(|data| Tensor::<B, 2>::from_data(data, &self.device))
            .map(|tensor| tensor.reshape([1, 28, 28]))
            .map(|tensor| ((tensor / 255) - 0.1307) / 0.3081)
            .collect();

        let targets = items
            .iter()
            .map(|item| {
                Tensor::<B, 1, Int>::from_data(
                    [(item.label as i64).elem::<B::IntElem>()],
                    &self.device,
                )
            })
            .collect();

        let images = Tensor::cat(images, 0).to_device(&self.device);
        let targets = Tensor::cat(targets, 0).to_device(&self.device);

        MnistBatch { images, targets }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct MnistTrainConfigSettings {
    pub num_epochs: Option<usize>,
    pub batch_size: Option<usize>,
    pub num_workers: Option<usize>,
    pub seed: Option<u64>,
    pub learning_rate: Option<f64>,
    pub num_classes: usize,
    pub hidden_size: usize,
    pub drop_out: Option<f64>,
}

impl ModelSettings for MnistTrainConfigSettings {}

#[derive(Config, TypedBuilder)]
pub struct MnistTrainingConfig {
    pub model: MnistConfig,
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
    type Settings = MnistTrainConfigSettings;

    fn with_settings(settings: &Self::Settings) -> Self {
        let mut model = MnistConfig::new(settings.num_classes, settings.hidden_size);
        if let Some(val) = settings.drop_out {
            model.dropout = val;
        }
        let optimizer = AdamConfig::new();
        let mut train_config = MnistTrainingConfig::new(model, optimizer);
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
pub struct MnistModel<B: Backend> {
    pub(crate) conv1: Conv2d<B>,
    pub(crate) conv2: Conv2d<B>,
    pub pool: AdaptiveAvgPool2d,
    pub dropout: Dropout,
    pub linear1: Linear<B>,
    pub linear2: Linear<B>,
    pub activation: Relu,
}

impl<B: Backend> MnistModel<B> {
    pub(crate) fn do_fedavg(
        mut global_model: MnistModel<B>,
        other_models: Vec<MnistModel<B>>,
        device: &B::Device,
    ) -> MnistModel<B> {
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
        global_model
    }

    fn get_average_tensor<const D: usize>(weights: Vec<Tensor<B, D>>) -> Tensor<B, D> {
        let mut avg_tensor = weights.get(0).expect("empty weights not possible").clone();
        let total_weights = weights.len() as f32;
        weights
            .into_iter()
            .skip(1)
            .for_each(|tensor| avg_tensor = avg_tensor.clone().add(tensor));
        avg_tensor.div_scalar(total_weights)
    }
}

#[derive(Config, Debug)]
pub struct MnistConfig {
    num_classes: usize,
    hidden_size: usize,
    #[config(default = "0.5")]
    dropout: f64,
}

impl MnistConfig {
    pub fn init<B: Backend>(&self, device: &B::Device) -> MnistModel<B> {
        MnistModel {
            conv1: Conv2dConfig::new([1, 8], [3, 3]).init(device),
            conv2: Conv2dConfig::new([8, 16], [3, 3]).init(device),
            pool: AdaptiveAvgPool2dConfig::new([8, 8]).init(),
            activation: Relu::new(),
            linear1: LinearConfig::new(16 * 8 * 8, self.hidden_size).init(device),
            linear2: LinearConfig::new(self.hidden_size, self.num_classes).init(device),
            dropout: DropoutConfig::new(self.dropout).init(),
        }
    }
}

impl<B: Backend> MnistModel<B> {
    pub fn forward(&self, images: Tensor<B, 3>) -> Tensor<B, 2> {
        let [batch_size, height, width] = images.dims();
        // Create a channel at the second dimension.
        let x = images.reshape([batch_size, 1, height, width]);

        let x = self.conv1.forward(x); // [batch_size, 8, _, _]
        let x = self.dropout.forward(x);
        let x = self.conv2.forward(x); // [batch_size, 16, _, _]
        let x = self.dropout.forward(x);
        let x = self.activation.forward(x);

        let x = self.pool.forward(x); // [batch_size, 16, 8, 8]
        let x = x.reshape([batch_size, 16 * 8 * 8]);
        let x = self.linear1.forward(x);
        let x = self.dropout.forward(x);
        let x = self.activation.forward(x);

        self.linear2.forward(x) // [batch_size, num_classes]
    }
}

impl<B: Backend> MnistModel<B> {
    pub fn forward_classification(
        &self,
        images: Tensor<B, 3>,
        targets: Tensor<B, 1, Int>,
    ) -> ClassificationOutput<B> {
        let output = self.forward(images);
        let loss =
            CrossEntropyLoss::new(None, &output.device()).forward(output.clone(), targets.clone());

        ClassificationOutput::new(loss, output, targets)
    }
}

impl<B: AutodiffBackend> TrainStep<MnistBatch<B>, ClassificationOutput<B>> for MnistModel<B> {
    fn step(&self, batch: MnistBatch<B>) -> TrainOutput<ClassificationOutput<B>> {
        let item = self.forward_classification(batch.images, batch.targets);
        TrainOutput::new(self, item.loss.backward(), item)
    }
}

impl<B: Backend> ValidStep<MnistBatch<B>, ClassificationOutput<B>> for MnistModel<B> {
    fn step(&self, batch: MnistBatch<B>) -> ClassificationOutput<B> {
        self.forward_classification(batch.images, batch.targets)
    }
}

pub fn mnist_train<B: AutodiffBackend>(
    output_path: &PathBuf,
    config: MnistTrainingConfig,
    test_data: MnistFlDataset,
    train_data: MnistFlDataset,
    current_model: MnistModel<B>,
    device: B::Device,
) -> ModelType<B> {
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

    let learner = LearnerBuilder::new(output_path.to_str().expect("invalid output path"))
        .metric_train_numeric(AccuracyMetric::new())
        .metric_valid_numeric(AccuracyMetric::new())
        .metric_train_numeric(LossMetric::new())
        .metric_valid_numeric(LossMetric::new())
        .with_file_checkpointer(CompactRecorder::new())
        .devices(vec![device.clone()])
        .num_epochs(config.num_epochs)
        .renderer(CustomRenderer {})
        .summary()
        .build(current_model, config.optimizer.init(), config.learning_rate);

    let updated_model = learner.fit(dataloader_train, dataloader_test);
    ModelType::Mnist(updated_model)
}
