use std::path::PathBuf;

use arrow::datatypes::{DataType, Field, Schema};
use hashbrown::{HashMap, HashSet};
use log::debug;

use disolv_core::bucket::TimeMS;
use disolv_input::columns::{AGENT_ID, OFF_TIMES, ON_TIMES};
use disolv_output::result::ResultWriter;
use disolv_output::writer::WriterType;

use crate::activation::cache::{ActivationCache, ActivationInfo};
use crate::simulation::config::ActivationSettings;
use crate::trace::cache::TraceInfo;

pub(crate) struct ActivationWriter {
    writer: WriterType,
    activations_cache: ActivationCache,
    active_vehicles: HashMap<u64, ActivationInfo>,
}

impl ResultWriter for ActivationWriter {
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

    fn close_file(self) {
        self.writer.close()
    }
}

impl ActivationWriter {
    pub(crate) fn new(activation_settings: &ActivationSettings) -> Self {
        let activation_file = PathBuf::from(activation_settings.activation_file.to_owned());
        let writer = WriterType::new(&activation_file, Self::schema());
        let cache_size = 10000;
        Self {
            writer,
            activations_cache: ActivationCache::new(cache_size),
            active_vehicles: HashMap::new(),
        }
    }

    pub(crate) fn determine_activations(&mut self, trace_data: &[TraceInfo], now: TimeMS) {
        trace_data.iter().for_each(|trace_info| {
            if self.active_vehicles.contains_key(&trace_info.agent_id) {
                // update the off time of this vehicle
                self.active_vehicles
                    .get_mut(&trace_info.agent_id)
                    .expect("failed to get")
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

        // Check vehicles that did not receive a new update
        self.active_vehicles.values_mut().for_each(|act_info| {
            if act_info.off_time < now.as_u64() {
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

    pub(crate) fn flush(mut self) {
        debug!(r"Activation parsing is done. Flushing the cache to file");
        self.mark_everything_complete();
        self.writer
            .record_batch_to_file(&self.activations_cache.as_record_batch());
        self.writer.close();
    }
}
