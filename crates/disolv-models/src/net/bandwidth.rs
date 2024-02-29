use crate::net::message::PayloadInfo;
use crate::net::metrics::Bandwidth;
use disolv_core::metrics::{Consumable, Feasibility, MetricSettings};
use log::error;
use serde::Deserialize;

#[serde_with::skip_serializing_none]
#[derive(Deserialize, Debug, Clone)]
pub struct BandwidthConfig {
    pub variant: String,
    pub constraint: Bandwidth,
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

    fn available(&self) -> Bandwidth {
        match self {
            BandwidthType::Constant(bandwidth) => bandwidth.available(),
        }
    }

    fn constraint(&self) -> Bandwidth {
        match self {
            BandwidthType::Constant(bandwidth) => bandwidth.constraint(),
        }
    }
}

/// A ConstantBandwidth is a bandwidth that is constant for all time steps.
/// It is defined by the available bandwidth and a limit.
#[derive(Debug, Clone)]
pub struct ConstantBandwidth {
    pub available: Bandwidth,
    pub constraint: Bandwidth,
}

impl Consumable<Bandwidth> for ConstantBandwidth {
    type P = PayloadInfo;
    type S = BandwidthConfig;

    fn with_settings(settings: Self::S) -> Self {
        Self {
            available: Bandwidth::default(),
            constraint: settings.constraint,
        }
    }

    fn reset(&mut self) {
        self.available = self.constraint;
    }

    fn consume(&mut self, metadata: &Self::P) -> Feasibility<Bandwidth> {
        let data_bytes = metadata.total_size;
        if data_bytes.as_u64() > self.available.as_u64() {
            Feasibility::Infeasible(Bandwidth::default())
        } else {
            self.available = Bandwidth::new(self.available.as_u64() - data_bytes.as_u64());
            Feasibility::Feasible(self.available)
        }
    }

    fn constraint(&self) -> Bandwidth {
        self.constraint
    }

    fn available(&self) -> Bandwidth {
        self.available
    }
}
