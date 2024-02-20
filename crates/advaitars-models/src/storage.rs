use advaitars_core::message::PayloadInfo;
use advaitars_core::metrics::Bytes;
use advaitars_engine::metrics::{Feasibility, MetricSettings, NonReplenishable};
use serde::Deserialize;

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct StorageConfig {
    pub variant: String,
    pub available: Bytes,
    pub limit: Bytes,
}

impl MetricSettings for StorageConfig {}

#[derive(Debug, Clone)]
pub enum StorageType {
    Constant(ConstantStorage),
}

#[derive(Debug, Clone)]
pub struct ConstantStorage {
    pub available: Bytes,
    pub limit: Bytes,
}

impl NonReplenishable<Bytes> for ConstantStorage {
    type P = PayloadInfo;
    type S = StorageConfig;

    fn with_settings(settings: Self::S) -> Self {
        Self {
            available: settings.available,
            limit: settings.limit,
        }
    }

    fn consume(&mut self, metadata: &Self::P) -> Feasibility<Bytes> {
        let consumed = self.available - metadata.total_size;
        if consumed < Bytes::new(0) {
            Feasibility::Infeasible(self.available)
        } else {
            self.available = consumed;
            Feasibility::Feasible(self.available)
        }
    }

    fn get_available(&self) -> Bytes {
        self.available
    }
}
