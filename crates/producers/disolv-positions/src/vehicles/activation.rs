use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::{ArrayRef, RecordBatch, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use hashbrown::HashMap;
use log::debug;
use typed_builder::TypedBuilder;

use disolv_input::columns::{AGENT_ID, OFF_TIMES, ON_TIMES};
use disolv_output::result::ResultWriter;
use disolv_output::writer::WriterType;

use crate::produce::config::TraceSettings;
use crate::vehicles::writer::TraceInfo;

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
    cache_limit: usize,
}

impl ActivationCache {
    pub(crate) fn new(cache_size: usize) -> Self {
        Self {
            agent_ids: Vec::with_capacity(cache_size),
            on_times: Vec::with_capacity(cache_size),
            off_times: Vec::with_capacity(cache_size),
            cache_limit: (cache_size * 90) / 100,
        }
    }

    pub(crate) fn is_full(&self) -> bool {
        self.agent_ids.len() >= self.cache_limit
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

pub(crate) struct VehicleActivations {
    writer: WriterType,
    activations_cache: ActivationCache,
    active_vehicles: HashMap<u64, ActivationInfo>,
}

impl ResultWriter for VehicleActivations {
    fn schema() -> Schema {
        let agent_id = Field::new(AGENT_ID, DataType::UInt64, false);
        let on_time = Field::new(ON_TIMES, DataType::UInt64, false);
        let off_time = Field::new(OFF_TIMES, DataType::UInt64, false);
        Schema::new(vec![agent_id, on_time, off_time])
    }

    fn write_to_file(&mut self) {
        if self.activations_cache.is_full() {
            self.writer
                .record_batch_to_file(&self.activations_cache.as_record_batch());
        }
    }

    fn close_file(mut self) {
        debug!(r"Activation parsing is done. Flushing the cache to file");
        self.mark_everything_complete();
        self.writer
            .record_batch_to_file(&self.activations_cache.as_record_batch());
        self.writer.close()
    }
}

impl VehicleActivations {
    pub(crate) fn new(trace_settings: &TraceSettings) -> Self {
        let activation_file = PathBuf::from(trace_settings.activation_file.to_owned());
        let writer = WriterType::new(&activation_file, Self::schema());
        let cache_size = 10000;
        Self {
            writer,
            activations_cache: ActivationCache::new(cache_size),
            active_vehicles: HashMap::new(),
        }
    }

    pub(crate) fn determine_activations(&mut self, trace_data: &[TraceInfo], now: u64) {
        trace_data.iter().for_each(|trace_info| {
            if self.active_vehicles.contains_key(&trace_info.agent_id) {
                // update the off time of this vehicle
                self.active_vehicles
                    .get_mut(&trace_info.agent_id)
                    .expect("failed to get agent")
                    .off_time = trace_info.time_ms;
            } else {
                // tracking a new vehicle
                self.active_vehicles.insert(
                    trace_info.agent_id,
                    ActivationInfo::builder()
                        .on_time(trace_info.time_ms)
                        .off_time(trace_info.time_ms)
                        .agent_id(trace_info.agent_id)
                        .is_complete(false)
                        .build(),
                );
            }
        });
        self.handle_stale_vehicles(now);
    }

    fn handle_stale_vehicles(&mut self, now: u64) {
        // Check vehicles that did not receive a new update
        self.active_vehicles.values_mut().for_each(|act_info| {
            if act_info.off_time < now {
                // mark it complete and push it to cache
                act_info.is_complete = true;
                self.activations_cache.append_activation(*act_info);
            }
        });

        // Remove completed traces from active vehicles map
        self.active_vehicles
            .retain(|_, act_info| !act_info.is_complete);
    }

    fn mark_everything_complete(&mut self) {
        self.active_vehicles.values_mut().for_each(|act_info| {
            // mark it complete and push it to cache
            act_info.is_complete = true;
            self.activations_cache.append_activation(*act_info);
        });
    }
}
