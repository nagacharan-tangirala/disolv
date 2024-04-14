use crate::net::message::PayloadInfo;
use crate::net::metrics::Bytes;
use disolv_core::metrics::{Feasibility, MetricSettings, Resource};
use serde::Deserialize;

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct StorageSettings {
    pub variant: String,
    pub limit: Bytes,
}

impl MetricSettings for StorageSettings {}

#[derive(Debug, Copy, Clone)]
pub enum StorageType {
    Constant(ConstantStorage),
}

impl Resource<Bytes> for StorageType {
    type P = PayloadInfo;
    type S = StorageSettings;

    fn with_settings(settings: &Self::S) -> Self {
        match settings.variant.to_lowercase().as_str() {
            "constant" => StorageType::Constant(ConstantStorage::with_settings(&settings)),
            _ => panic!("Unsupported storage type {}", settings.variant),
        }
    }

    fn consume(&mut self, metadata: &Self::P) -> Feasibility<Bytes> {
        match self {
            StorageType::Constant(storage) => storage.consume(metadata),
        }
    }

    fn available(&self) -> Bytes {
        match self {
            StorageType::Constant(storage) => storage.available(),
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct ConstantStorage {
    pub available: Bytes,
    limit: Bytes,
}

impl Resource<Bytes> for ConstantStorage {
    type P = PayloadInfo;
    type S = StorageSettings;

    fn with_settings(settings: &StorageSettings) -> Self {
        Self {
            limit: settings.limit,
            ..Default::default()
        }
    }

    fn consume(&mut self, metadata: &Self::P) -> Feasibility<Bytes> {
        let to_consume = self.available - metadata.total_size;
        if self.available + to_consume > self.limit {
            Feasibility::Infeasible(self.available)
        } else {
            self.available += to_consume;
            Feasibility::Feasible(self.available)
        }
    }

    fn available(&self) -> Bytes {
        self.available
    }
}
