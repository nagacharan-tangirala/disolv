use std::path::PathBuf;

use arrow::datatypes::{DataType, Field, Schema};
use log::debug;

use disolv_input::columns::{AGENT_ID, OFF_TIMES, ON_TIMES};
use disolv_output::result::ResultWriter;
use disolv_output::writer::WriterType;

use crate::produce::config::RSUSettings;
use crate::vehicles::activation::{ActivationCache, ActivationInfo};

pub(crate) struct RSUActivations {
    writer: WriterType,
    activations_cache: ActivationCache,
}

impl ResultWriter for RSUActivations {
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
        self.writer
            .record_batch_to_file(&self.activations_cache.as_record_batch());
        self.writer.close()
    }
}

impl RSUActivations {
    pub(crate) fn new(rsu_settings: &RSUSettings) -> Self {
        let activation_file = PathBuf::from(rsu_settings.activation_file.to_owned());
        let writer = WriterType::new(&activation_file, Self::schema());
        let cache_size = 10000;
        Self {
            writer,
            activations_cache: ActivationCache::new(cache_size),
        }
    }

    pub(crate) fn store_activation(&mut self, agent_id: u64, duration: u64) {
        let activation = ActivationInfo::builder()
            .on_time(0)
            .off_time(duration)
            .agent_id(agent_id)
            .is_complete(true)
            .build();
        self.activations_cache.append_activation(activation);
    }
}
