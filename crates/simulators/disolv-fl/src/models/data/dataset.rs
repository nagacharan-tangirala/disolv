use burn::data::dataset::Dataset;

use crate::models::data::cifar::CifarFlDataset;
use crate::models::data::mnist::MnistFlDataset;

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
            DatasetType::Cifar(cifar) => cifar.len(),
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
            DatasetType::Cifar(cifar) => !cifar.images.is_empty(),
        }
    }

    pub fn clear(&mut self) {
        match self {
            DatasetType::Mnist(mnist) => mnist.images.clear(),
            DatasetType::Cifar(cifar) => cifar.images.clear(),
        }
    }
}
