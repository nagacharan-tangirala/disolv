use std::path::PathBuf;

use burn::config::Config;
use burn::data::dataloader::{DataLoaderBuilder, Dataset};
use burn::data::dataloader::batcher::Batcher;
use burn::data::dataset::vision::{MnistDataset, MnistItem};
use burn::module::Module;
use burn::nn::{Dropout, DropoutConfig, Linear, LinearConfig, Relu};
use burn::nn::conv::{Conv2d, Conv2dConfig};
use burn::nn::loss::CrossEntropyLoss;
use burn::nn::pool::{AdaptiveAvgPool2d, AdaptiveAvgPool2dConfig};
use burn::optim::AdamConfig;
use burn::prelude::{Backend, Tensor};
use burn::record::CompactRecorder;
use burn::tensor::{Data, ElementConversion, Int};
use burn::tensor::backend::AutodiffBackend;
use burn::train::{ClassificationOutput, LearnerBuilder, TrainOutput, TrainStep, ValidStep};
use burn::train::metric::{AccuracyMetric, LossMetric};
use typed_builder::TypedBuilder;

use crate::models::ai::models::BatchType;
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
            .map(|item| Data::<f32, 2>::from(item.image))
            .map(|data| Tensor::<B, 2>::from_data(data.convert(), &self.device))
            .map(|tensor| tensor.reshape([1, 28, 28]))
            .map(|tensor| ((tensor / 255) - 0.1307) / 0.3081)
            .collect();

        let targets = items
            .iter()
            .map(|item| {
                Tensor::<B, 1, Int>::from_data(
                    Data::from([(item.label as i64).elem()]),
                    &self.device,
                )
            })
            .collect();

        let images = Tensor::cat(images, 0).to_device(&self.device);
        let targets = Tensor::cat(targets, 0).to_device(&self.device);

        MnistBatch { images, targets }
    }
}

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

#[derive(Module, Debug)]
pub struct MnistModel<B: Backend> {
    pub conv1: Conv2d<B>,
    pub conv2: Conv2d<B>,
    pub pool: AdaptiveAvgPool2d,
    pub dropout: Dropout,
    pub linear1: Linear<B>,
    pub linear2: Linear<B>,
    pub activation: Relu,
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

#[derive(Clone, TypedBuilder)]
pub struct MnistTrainer<B>
where
    B: AutodiffBackend,
{
    device: B::Device,
    output_path: PathBuf,
    model: MnistModel<B>,
    config: MnistTrainingConfig,
}

impl<B: AutodiffBackend> MnistTrainer<B> {
    pub fn train(&mut self, test_data: MnistFlDataset, train_data: MnistFlDataset) {
        let batcher_train = MnistBatcher::<B>::new(self.device.clone());
        let batcher_valid = MnistBatcher::<B::InnerBackend>::new(self.device.clone());

        let dataloader_train = DataLoaderBuilder::new(batcher_train)
            .batch_size(self.config.batch_size)
            .shuffle(self.config.seed)
            .num_workers(self.config.num_workers)
            .build(train_data);

        let dataloader_test = DataLoaderBuilder::new(batcher_valid)
            .batch_size(self.config.batch_size)
            .shuffle(self.config.seed)
            .num_workers(self.config.num_workers)
            .build(test_data);

        let learner = LearnerBuilder::new(
            self.output_path
                .clone()
                .to_str()
                .expect("invalid output path"),
        )
        .metric_train_numeric(AccuracyMetric::new())
        .metric_valid_numeric(AccuracyMetric::new())
        .metric_train_numeric(LossMetric::new())
        .metric_valid_numeric(LossMetric::new())
        .with_file_checkpointer(CompactRecorder::new())
        .devices(vec![self.device.clone()])
        .num_epochs(self.config.num_epochs)
        .renderer(CustomRenderer {})
        .summary()
        .build(
            self.config.model.init::<B>(&self.device),
            self.config.optimizer.init(),
            self.config.learning_rate,
        );

        let model_trained = learner.fit(dataloader_train, dataloader_test);

        model_trained
            .save_file(
                self.output_path
                    .clone()
                    .join("model")
                    .to_str()
                    .expect("invalid output path"),
                &CompactRecorder::new(),
            )
            .expect("Trained model should be saved successfully");
    }
}
