use burn::data::dataset::Dataset;
use burn::data::dataset::vision::MnistItem;
use burn::prelude::Backend;
use typed_builder::TypedBuilder;

use crate::models::data::cifar::CifarFlDataset;
use crate::models::data::mnist::MnistFlDataset;

#[derive(Clone, TypedBuilder)]
pub struct SampleBatcher<B: Backend> {
    pub device: B::Device,
}

impl<B: Backend> SampleBatcher<B> {
    pub fn new(device: B::Device) -> Self {
        Self { device }
    }
}

#[derive(Clone, Debug)]
pub enum DatasetType {
    Mnist(MnistFlDataset),
    Cifar(CifarFlDataset),
}

impl DatasetType {
    pub fn from_str(dataset_type: &str) -> Self {
        match dataset_type.to_lowercase().as_str() {
            "mnist" => DatasetType::Mnist(MnistFlDataset::default()),
            "cifar" => DatasetType::Cifar(CifarFlDataset::default()),
            _ => unimplemented!("{} dataset is not implemented", dataset_type),
        }
    }

    pub fn data_length(&self) -> usize {
        match self {
            DatasetType::Mnist(mnist) => mnist.len(),
            _ => 0,
        }
    }
    pub fn dataset_type(&self) -> &str {
        match self {
            DatasetType::Mnist(_) => "mnist",
            DatasetType::Cifar(_) => "cifar",
        }
    }

    pub fn has_data(&self) -> bool {
        match self {
            DatasetType::Mnist(mnist) => !mnist.images.is_empty(),
            DatasetType::Cifar(_) => false,
        }
    }

    pub fn clear(&mut self) {
        match self {
            DatasetType::Mnist(mnist) => mnist.images.clear(),
            _ => unimplemented!("Unable to clear dataset"),
        }
    }
}
