use crate::result::{OutputSettings, OutputType};
use crate::writer::DataOutput;
use arrow::array::{ArrayRef, RecordBatch, UInt32Array, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use disolv_core::bucket::TimeMS;
use disolv_core::metrics::Consumable;
use disolv_models::net::slice::Slice;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct NetStatWriter {
    time_step: Vec<u64>,
    slice_id: Vec<u32>,
    bandwidth: Vec<u64>,
    to_output: DataOutput,
}

impl NetStatWriter {
    pub fn new(output_settings: &OutputSettings) -> Self {
        let output_path = PathBuf::from(&output_settings.output_path);
        let config = output_settings
            .file_out_config
            .iter()
            .find(|&file_out_config| file_out_config.output_type == OutputType::NetStat)
            .expect("NetStatWriter::new: No NetStatWriter config found");
        let output_file = output_path.join(&config.output_filename);

        Self {
            to_output: DataOutput::new(&output_file, Self::schema()),
            time_step: Vec::new(),
            slice_id: Vec::new(),
            bandwidth: Vec::new(),
        }
    }

    fn schema() -> Schema {
        let time_ms = Field::new("time_step", DataType::UInt64, false);
        let slice_id = Field::new("slice_id", DataType::UInt32, false);
        let bandwidth = Field::new("bandwidth", DataType::UInt64, false);
        Schema::new(vec![time_ms, slice_id, bandwidth])
    }

    pub fn add_data(&mut self, time_step: TimeMS, slice: &Slice) {
        self.time_step.push(time_step.as_u64());
        self.slice_id.push(slice.id);
        self.bandwidth
            .push(slice.resources.bandwidth_type.available().as_u64());
    }

    pub fn write_to_file(&mut self) {
        match &mut self.to_output {
            DataOutput::Parquet(to_output) => {
                let record_batch = RecordBatch::try_from_iter(vec![
                    (
                        "time_step",
                        Arc::new(UInt64Array::from(std::mem::take(&mut self.time_step)))
                            as ArrayRef,
                    ),
                    (
                        "slice_id",
                        Arc::new(UInt32Array::from(std::mem::take(&mut self.slice_id))) as ArrayRef,
                    ),
                    (
                        "bandwidth",
                        Arc::new(UInt64Array::from(std::mem::take(&mut self.bandwidth)))
                            as ArrayRef,
                    ),
                ])
                .expect("Failed to convert results to record batch");
                to_output
                    .writer
                    .write(&record_batch)
                    .expect("Failed to write record batches to file");
            }
        }
    }

    pub(crate) fn close_files(self) {
        match self.to_output {
            DataOutput::Parquet(to_output) => to_output.close(),
        }
    }
}
