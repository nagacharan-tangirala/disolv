use log::error;
use pavenet_core::message::PayloadInfo;
use pavenet_core::metrics::Bandwidth;
use pavenet_engine::metrics::{Consumable, Feasibility, Measurable, Metric, MetricSettings};
use serde::Deserialize;

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct BandwidthConfig {
    pub variant: String,
    pub available: Bandwidth,
    pub limit: Bandwidth,
}

impl MetricSettings for BandwidthConfig {}

#[derive(Debug, Clone)]
pub enum BandwidthType {
    Constant(ConstantBandwidth),
}

impl Consumable<Bandwidth> for BandwidthType {
    type P = PayloadInfo;
    type S = BandwidthConfig;

    fn with_settings(settings: Self::S) -> Self {
        match settings.variant.to_lowercase().as_str() {
            "constant" => BandwidthType::Constant(ConstantBandwidth::with_settings(settings)),
            _ => {
                error!("Only Constant bandwidth variant is supported.");
                panic!("Unsupported bandwidth variant {}.", settings.variant);
            }
        }
    }

    fn reset(&mut self) {
        match self {
            BandwidthType::Constant(bandwidth) => bandwidth.reset(),
        }
    }

    fn consume(&mut self, metadata: &Self::P) -> Feasibility<Bandwidth> {
        match self {
            BandwidthType::Constant(bandwidth) => bandwidth.consume(metadata),
        }
    }

    fn get_available(&self) -> Bandwidth {
        match self {
            BandwidthType::Constant(bandwidth) => bandwidth.get_available(),
        }
    }
}

/// A ConstantBandwidth is a bandwidth that is constant for all time steps.
/// It is defined by the available bandwidth and a limit.
#[derive(Debug, Clone)]
pub struct ConstantBandwidth {
    pub available: Bandwidth,
    pub limit: Bandwidth,
}

impl Consumable<Bandwidth> for ConstantBandwidth {
    type P = PayloadInfo;
    type S = BandwidthConfig;

    fn with_settings(settings: Self::S) -> Self {
        Self {
            available: Bandwidth::default(),
            limit: settings.limit,
        }
    }

    fn reset(&mut self) {
        self.available = Bandwidth::default();
    }

    fn consume(&mut self, metadata: &Self::P) -> Feasibility<Bandwidth> {
        let data_bytes = metadata.total_size;
        if data_bytes > self.available.as_f32() {
            Feasibility::Infeasible(Bandwidth::default())
        } else {
            self.available -= data_bytes;
            Feasibility::Feasible(self.available)
        }
    }

    fn get_available(&self) -> Bandwidth {
        self.available
    }
}
