use crate::net::bandwidth::{BandwidthConfig, BandwidthType};
use crate::net::latency::{LatencyConfig, LatencyType};
use crate::net::message::{DPayload, TxMetrics};
use disolv_core::bucket::TimeMS;
use disolv_core::metrics::{Consumable, Measurable};
use serde::Deserialize;
use typed_builder::TypedBuilder;

#[derive(Deserialize, Debug, Clone)]
pub struct SliceSettings {
    pub id: u32,
    pub name: String,
    pub latency: LatencyConfig,
    pub bandwidth: BandwidthConfig,
}

#[derive(Clone, Debug, TypedBuilder)]
pub struct RadioMetrics {
    pub latency_type: LatencyType,
}

#[derive(Clone, Debug, TypedBuilder)]
pub struct RadioResources {
    pub bandwidth_type: BandwidthType,
}

#[derive(Clone, Debug, TypedBuilder)]
pub struct Slice {
    pub id: u32,
    pub name: String,
    pub metrics: RadioMetrics,
    pub resources: RadioResources,
    pub step_size: TimeMS,
    #[builder(default)]
    pub tx_order: u32,
}

impl Slice {
    pub fn reset(&mut self) {
        self.tx_order = 0;
        self.resources.bandwidth_type.reset();
    }

    pub fn transfer(&mut self, payload: &DPayload) -> TxMetrics {
        self.tx_order += 1;
        let mut tx_metrics = TxMetrics::new(payload, self.tx_order);
        tx_metrics.latency = self
            .metrics
            .latency_type
            .measure(&tx_metrics, &payload.metadata);

        tx_metrics.bandwidth = self.resources.bandwidth_type.consume(&payload.metadata);
        tx_metrics
    }
}
