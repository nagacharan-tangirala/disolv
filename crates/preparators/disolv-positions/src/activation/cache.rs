use std::sync::Arc;

use arrow::array::{ArrayRef, RecordBatch, UInt64Array};
use typed_builder::TypedBuilder;

use disolv_input::columns::{AGENT_ID, OFF_TIMES, ON_TIMES};

#[derive(Copy, Clone, TypedBuilder)]
pub(crate) struct ActivationInfo {
    pub(crate) on_time: u64,
    pub(crate) agent_id: u64,
    pub(crate) off_time: u64,
    pub(crate) is_complete: bool,
}

pub(crate) struct ActivationCache {
    agent_ids: Vec<u64>,
    on_times: Vec<u64>,
    off_times: Vec<u64>,
    cache_size: usize,
}

impl ActivationCache {
    pub(crate) fn new(cache_size: usize) -> Self {
        Self {
            agent_ids: Vec::with_capacity(cache_size),
            on_times: Vec::with_capacity(cache_size),
            off_times: Vec::with_capacity(cache_size),
            cache_size,
        }
    }

    pub(crate) fn is_full(&self) -> bool {
        self.agent_ids.len() >= self.cache_size
    }

    pub(crate) fn append_activation(&mut self, activation_info: ActivationInfo) {
        self.agent_ids.push(activation_info.agent_id);
        self.on_times.push(activation_info.on_time);
        self.off_times.push(activation_info.off_time);
    }

    pub(crate) fn as_record_batch(&mut self) -> RecordBatch {
        RecordBatch::try_from_iter(vec![
            (
                AGENT_ID,
                Arc::new(UInt64Array::from(std::mem::take(&mut self.agent_ids))) as ArrayRef,
            ),
            (
                ON_TIMES,
                Arc::new(UInt64Array::from(std::mem::take(&mut self.on_times))) as ArrayRef,
            ),
            (
                OFF_TIMES,
                Arc::new(UInt64Array::from(std::mem::take(&mut self.off_times))) as ArrayRef,
            ),
        ])
        .expect("Failed to convert writer cache to record batch")
    }
}
