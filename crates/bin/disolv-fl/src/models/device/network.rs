use serde::Deserialize;
use typed_builder::TypedBuilder;

use disolv_core::bucket::TimeMS;
use disolv_models::net::network::{NetworkSlice, SliceType};

use crate::fl::device::DeviceInfo;
use crate::models::device::message::{
    FlPayload, FlPayloadInfo, Message, MessageType, MessageUnit, TxMetrics,
};

#[derive(Deserialize, Debug, Clone)]
pub struct SliceSettings {
    pub id: u32,
    pub name: FlSlice,
}

#[derive(Clone, Debug, TypedBuilder)]
pub struct RadioMetrics {}

#[derive(Clone, Debug, TypedBuilder)]
pub struct RadioResources {}

#[derive(Deserialize, Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum FlSlice {
    #[default]
    Wireless5G,
}

impl SliceType for FlSlice {}

#[derive(Clone, Debug, TypedBuilder)]
pub struct Slice {
    pub id: u32,
    pub metrics: RadioMetrics,
    pub resources: RadioResources,
    pub step_size: TimeMS,
    #[builder(default)]
    pub tx_order: u32,
}

impl NetworkSlice<MessageType, MessageUnit, FlPayloadInfo, DeviceInfo, Message, TxMetrics>
    for Slice
{
    fn reset(&mut self) {
        self.tx_order = 0;
    }

    fn transfer(&mut self, payload: &FlPayload) -> TxMetrics {
        self.tx_order += 1;
        let mut tx_metrics = TxMetrics::new(&payload, self.tx_order);

        // match self.resources.bandwidth_type.consume(&payload.metadata) {
        //     Feasibility::Feasible(bandwidth) => tx_metrics.bandwidth = bandwidth,
        //     Feasibility::Infeasible(available) => {
        //         tx_metrics.bandwidth = available;
        //         tx_metrics.tx_status = TxStatus::Fail;
        //         tx_metrics.tx_fail_reason = TxFailReason::NoBandwidth;
        //         return tx_metrics;
        //     }
        // };
        tx_metrics
    }
}
