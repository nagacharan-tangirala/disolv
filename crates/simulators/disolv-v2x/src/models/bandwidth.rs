use serde::Deserialize;

use disolv_core::message::Metadata;
use disolv_core::metrics::{Consumable, Feasibility, MetricSettings};
use disolv_models::net::metrics::Bandwidth;

use crate::models::message::PayloadInfo;

#[derive(Deserialize, Debug, Clone)]
pub struct BandwidthConfig {
    pub variant: String,
}

impl MetricSettings for BandwidthConfig {}

#[derive(Debug, Clone)]
pub enum BandwidthType {
    Constant(ConstantBandwidth),
}

impl Consumable<Bandwidth, PayloadInfo> for BandwidthType {
    type S = BandwidthConfig;

    fn with_settings(settings: Self::S) -> Self {
        match settings.variant.to_lowercase().as_str() {
            "constant" => BandwidthType::Constant(ConstantBandwidth::with_settings(settings)),
            _ => panic!("Unsupported bandwidth variant {}.", settings.variant),
        }
    }

    fn reset(&mut self) {
        match self {
            Self::Constant(constant) => constant.reset(),
        }
    }

    fn consume(&mut self, metadata: &PayloadInfo) -> Feasibility<Bandwidth> {
        match self {
            Self::Constant(constant) => constant.consume(metadata),
        }
    }

    fn available(&self) -> Bandwidth {
        match self {
            Self::Constant(constant) => constant.available(),
        }
    }
}

/// A ConstantBandwidth is a bandwidth that is constant for all time steps.
#[derive(Debug, Clone, Default)]
pub struct ConstantBandwidth {
    pub bandwidth: Bandwidth,
}

impl Consumable<Bandwidth, PayloadInfo> for ConstantBandwidth {
    type S = BandwidthConfig;

    fn with_settings(settings: Self::S) -> Self {
        Self {
            ..Default::default()
        }
    }

    fn reset(&mut self) {
        self.bandwidth = Bandwidth::default();
    }

    fn consume(&mut self, metadata: &PayloadInfo) -> Feasibility<Bandwidth> {
        let data_bytes = metadata.size();
        self.bandwidth = Bandwidth::new(10000);
        Feasibility::Feasible(self.bandwidth)
    }

    fn available(&self) -> Bandwidth {
        self.bandwidth
    }
}
