use serde::Deserialize;
use typed_builder::TypedBuilder;

use disolv_core::bucket::TimeMS;
use disolv_core::metrics::{Consumable, Feasibility, Measurable};
use disolv_models::net::network::{NetworkSlice, SliceType};

use crate::models::bandwidth::{BandwidthConfig, BandwidthType};
use crate::models::latency::{LatencyConfig, LatencyType};
use crate::models::message::{
    DataBlob, DataType, MessageType, PayloadInfo, TxFailReason, TxMetrics, TxStatus, V2XPayload,
};
use crate::v2x::device::DeviceInfo;

#[derive(Deserialize, Debug, Clone)]
pub struct SliceSettings {
    pub id: u32,
    pub name: V2XSlice,
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

#[derive(Deserialize, Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum V2XSlice {
    #[default]
    Wireless5G,
}

impl SliceType for V2XSlice {}

#[derive(Clone, Debug, TypedBuilder)]
pub struct Slice {
    pub id: u32,
    pub metrics: RadioMetrics,
    pub resources: RadioResources,
    pub step_size: TimeMS,
    #[builder(default)]
    pub tx_order: u32,
}

impl NetworkSlice<DataType, DataBlob, PayloadInfo, DeviceInfo, MessageType, TxMetrics> for Slice {
    fn reset(&mut self) {
        self.tx_order = 0;
        self.resources.bandwidth_type.reset();
    }

    fn transfer(&mut self, payload: &V2XPayload) -> TxMetrics {
        self.tx_order += 1;
        let mut tx_metrics = TxMetrics::new(&payload, self.tx_order);
        match self.metrics.latency_type.measure(&payload.metadata) {
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
