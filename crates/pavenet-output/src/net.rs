use crate::result::{OutputSettings, OutputType};
use crate::writer::DataOutput;
use pavenet_core::metrics::Bandwidth;
use pavenet_engine::bucket::{Resultant, TimeMS};
use pavenet_engine::metrics::Consumable;
use pavenet_models::slice::Slice;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Serialize)]
struct NetStats {
    pub(crate) time_step: u32,
    pub(crate) slice_id: u32,
    pub(crate) bandwidth: Bandwidth,
    pub(crate) bandwidth_used: Bandwidth,
}

impl Resultant for NetStats {}

impl NetStats {
    fn from_data(time_step: TimeMS, slice: &Slice) -> Self {
        Self {
            time_step: time_step.as_u32(),
            slice_id: slice.id,
            bandwidth: slice.resources.bandwidth_type.constraint(),
            bandwidth_used: slice.resources.bandwidth_type.available(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NetStatWriter {
    net_stats: Vec<NetStats>,
    to_output: DataOutput,
}

impl NetStatWriter {
    pub fn new(output_settings: &OutputSettings) -> Self {
        let output_path = PathBuf::from(&output_settings.output_path);
        let config = output_settings
            .file_out_config
            .iter()
            .find(|&file_out_config| file_out_config.output_type == OutputType::NetStat)
            .expect("NetStatWriter::new: No NetDataWriter config found");
        let output_file = output_path.join(&config.output_filename);
        Self {
            net_stats: Vec::new(),
            to_output: DataOutput::new(&output_file),
        }
    }

    pub fn add_data(&mut self, time_step: TimeMS, slice: &Slice) {
        self.net_stats.push(NetStats::from_data(time_step, slice));
    }

    pub fn write_to_file(&mut self) {
        self.to_output.write_to_file(&self.net_stats);
        self.net_stats.clear();
    }
}
