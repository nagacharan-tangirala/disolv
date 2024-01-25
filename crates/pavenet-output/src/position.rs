use crate::result::{OutputSettings, OutputType};
use crate::writer::DataOutput;
use log::debug;
use pavenet_core::mobility::MapState;
use pavenet_engine::bucket::{Resultant, TimeMS};
use pavenet_engine::node::NodeId;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Default, Clone, Copy, Debug, Serialize)]
pub struct NodePosition {
    pub(crate) time_step: u32,
    pub(crate) node_id: u32,
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl Resultant for NodePosition {}

impl NodePosition {
    pub fn from_data(time_step: TimeMS, node_id: NodeId, map_state: &MapState) -> Self {
        Self {
            time_step: time_step.as_u32(),
            node_id: node_id.as_u32(),
            x: map_state.pos.x,
            y: map_state.pos.y,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PosWriter {
    data_pos: Vec<NodePosition>,
    to_output: DataOutput,
}

impl PosWriter {
    pub fn new(output_settings: &OutputSettings) -> Self {
        let output_path = PathBuf::from(&output_settings.output_path);
        let config = output_settings
            .file_out_config
            .iter()
            .find(|&file_out_config| file_out_config.output_type == OutputType::NodePos)
            .expect("PosWriter::new: No PosWriter config found");
        let output_file = output_path.join(&config.output_filename);
        Self {
            data_pos: Vec::new(),
            to_output: DataOutput::new(&output_file),
        }
    }

    pub fn add_data(&mut self, time_step: TimeMS, node_id: NodeId, map_state: &MapState) {
        self.data_pos
            .push(NodePosition::from_data(time_step, node_id, map_state));
    }

    pub fn write_to_file(&mut self) {
        debug!("Writing positions to file");
        self.to_output
            .write_to_file(std::mem::take(&mut self.data_pos));
    }
}
