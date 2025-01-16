use burn::data::dataloader::batcher::Batcher;
use burn::data::dataloader::Dataset;
use burn::data::dataset::vision::{MnistDataset, MnistItem};
use burn::prelude::{Backend, ElementConversion, Int, Tensor, TensorData};

use crate::models::ai::common::BatchType;
use crate::models::data::dataset::SampleBatcher;

#[derive(Default, Clone, Debug)]
pub struct MnistFlDataset {
    pub images: Vec<MnistItem>,
}

impl MnistFlDataset {
    pub fn new(batch_type: BatchType) -> Self {
        let images = match batch_type {
            BatchType::Test => MnistDataset::test().iter().collect(),
            BatchType::Train => MnistDataset::train().iter().collect(),
        };
        Self { images }
    }

    pub fn with_images(images: Vec<MnistItem>) -> Self {
        Self { images }
    }

    pub fn split_with_ratios(ratios: Vec<f64>, batch_type: BatchType) -> Vec<MnistFlDataset> {
        let all_data = MnistFlDataset::new(batch_type).to_owned();
        let total_samples = all_data.len();
        let mut data_chunks = Vec::new();
        let chunk_sizes: Vec<usize> = ratios
            .iter()
            .map(|r| (r * total_samples as f64).round() as usize)
            .collect();

        let mut start = 0;
        for size in chunk_sizes {
            if start >= total_samples {
                break;
            }
            let end = usize::min(start + size, total_samples);
            let chunk = all_data.images[start..end].to_vec();
            data_chunks.push(MnistFlDataset::with_images(chunk));
            start = end;
        }
        data_chunks
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

#[derive(Clone, Debug)]
pub struct MnistBatch<B: Backend> {
    pub images: Tensor<B, 3>,
    pub targets: Tensor<B, 1, Int>,
}

impl<B: Backend> Batcher<MnistItem, MnistBatch<B>> for SampleBatcher<B> {
    fn batch(&self, items: Vec<MnistItem>) -> MnistBatch<B> {
        let images = items
            .iter()
            .map(|item| TensorData::from(item.image))
            .map(|data| Tensor::<B, 2>::from_data(data.convert::<B::FloatElem>(), &self.device))
            .map(|tensor| tensor.reshape([1, 28, 28]))
            .map(|tensor| ((tensor / 255) - 0.1307) / 0.3081)
            .collect();

        let targets = items
            .iter()
            .map(|item| {
                Tensor::<B, 1, Int>::from_data(
                    TensorData::from([(item.label as i64).elem::<B::IntElem>()]),
                    &self.device,
                )
            })
            .collect();

        let images = Tensor::cat(images, 0);
        let targets = Tensor::cat(targets, 0);

        MnistBatch { images, targets }
    }
}
