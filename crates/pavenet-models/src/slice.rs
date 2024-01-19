use crate::bandwidth::{BandwidthConfig, BandwidthType};
use crate::latency::{LatencyConfig, LatencyType};
use pavenet_core::message::{DPayload, TxFailReason, TxMetrics, TxStatus};
use pavenet_engine::bucket::TimeMS;
use pavenet_engine::metrics::{Consumable, Feasibility, Measurable};
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
        match self
            .metrics
            .latency_type
            .measure(&tx_metrics, &payload.metadata)
        {
            Feasibility::Feasible(latency) => tx_metrics.latency = latency,
            Feasibility::Infeasible(latency) => {
                tx_metrics.latency = latency;
                tx_metrics.tx_status = TxStatus::Fail;
                tx_metrics.tx_fail_reason = TxFailReason::LatencyLimit;
                return tx_metrics;
            }
        };

        match self.resources.bandwidth_type.consume(&payload.metadata) {
            Feasibility::Feasible(bandwidth) => tx_metrics.bandwidth = bandwidth,
            Feasibility::Infeasible(available) => {
                tx_metrics.bandwidth = available;
                tx_metrics.tx_status = TxStatus::Fail;
                tx_metrics.tx_fail_reason = TxFailReason::NoBandwidth;
                return tx_metrics;
            }
        };

        tx_metrics.tx_status = TxStatus::Ok;
        tx_metrics
    }
}
