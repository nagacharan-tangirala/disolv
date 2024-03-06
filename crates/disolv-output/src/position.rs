use crate::result::{OutputSettings, OutputType};
use crate::writer::DataOutput;
use disolv_core::agent::AgentId;
use disolv_core::bucket::{Resultant, TimeMS};
use disolv_models::device::mobility::MapState;
use log::debug;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Default, Clone, Copy, Debug, Serialize)]
pub struct AgentPosition {
    pub(crate) time_step: u32,
    pub(crate) agent_id: u32,
    pub(crate) x: f64,
    pub(crate) y: f64,
}

impl Resultant for AgentPosition {}

impl AgentPosition {
    pub fn from_data(time_step: TimeMS, agent_id: AgentId, map_state: &MapState) -> Self {
        Self {
            time_step: time_step.as_u32(),
            agent_id: agent_id.as_u32(),
            x: map_state.pos.x,
            y: map_state.pos.y,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PosWriter {
    data_pos: Vec<AgentPosition>,
    to_output: DataOutput,
}

impl PosWriter {
    pub fn new(output_settings: &OutputSettings) -> Self {
        let output_path = PathBuf::from(&output_settings.output_path);
        let config = output_settings
            .file_out_config
            .iter()
            .find(|&file_out_config| file_out_config.output_type == OutputType::AgentPos)
            .expect("PosWriter::new: No PosWriter config found");
        let output_file = output_path.join(&config.output_filename);
        Self {
            data_pos: Vec::new(),
            to_output: DataOutput::new(&output_file),
        }
    }

    pub fn add_data(&mut self, time_step: TimeMS, agent_id: AgentId, map_state: &MapState) {
        self.data_pos
            .push(AgentPosition::from_data(time_step, agent_id, map_state));
    }

    pub fn write_to_file(&mut self) {
        debug!("Writing positions to file");
        self.to_output
            .write_to_file(std::mem::take(&mut self.data_pos));
    }
}
