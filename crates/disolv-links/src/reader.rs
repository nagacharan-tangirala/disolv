use crate::config::PositionFiles;
use arrow_array::RecordBatch;
use disolv_core::agent::AgentId;
use disolv_core::bucket::TimeMS;
use disolv_input::batch::{read_f64_column, read_u64_column};
use disolv_input::columns::{AGENT_ID, COORD_X, COORD_Y, TIME_STEP};
use hashbrown::HashMap;
use kiddo::KdTree;
use log::debug;
use parquet::arrow::arrow_reader::{ParquetRecordBatchReader, ParquetRecordBatchReaderBuilder};
use serde::Deserialize;
use std::fs::File;
use std::path::PathBuf;

pub type AgentIdPos = Vec<(AgentId, [f64; 2])>;
pub type PositionMap = HashMap<TimeMS, AgentIdPos>;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize)]
pub(crate) enum TraceType {
    Constant = 0,
    Mobile,
}

#[derive(Debug)]
pub(crate) struct Reader {
    pub(crate) file_path: PathBuf,
    pub(crate) current_row_group: usize,
    pub(crate) positions: PositionMap,
    pub(crate) trace_type: TraceType,
    pub(crate) kd_tree: KdTree<f64, 2>,
}

impl Reader {
    pub(crate) fn new(position_files: &PositionFiles) -> Self {
        Self {
            file_path: PathBuf::from(position_files.position_file.to_owned()),
            trace_type: position_files.trace_type,
            positions: PositionMap::default(),
            current_row_group: usize::default(),
            kd_tree: KdTree::default(),
        }
    }

    pub(crate) fn read_constant_positions(&mut self) {
        assert_eq!(self.trace_type, TraceType::Constant);
        self.read_positions();
        if let Some(positions) = self.positions.get(&TimeMS::default()) {
            positions.iter().for_each(|pos| {
                self.kd_tree.add(&pos.1, pos.0.as_u64());
            });
        };
    }

    pub(crate) fn read_dynamic_positions_at(&mut self, time_ms: TimeMS) {
        assert_eq!(self.trace_type, TraceType::Mobile);
        if !self.positions.contains_key(&time_ms) {
            debug!("Reading positions at {}", time_ms);
            self.read_positions();
        }
        self.kd_tree = KdTree::default();
        if let Some(positions) = self.positions.get(&time_ms) {
            positions.iter().for_each(|pos| {
                self.kd_tree.add(&pos.1, pos.0.as_u64());
            });
        };
    }

    pub(crate) fn get_positions_at(&self, time_ms: TimeMS) -> Option<&AgentIdPos> {
        if self.trace_type == TraceType::Constant {
            return self.positions.get(&TimeMS::default());
        }
        self.positions.get(&time_ms)
    }

    pub(crate) fn get_position_tree(&self) -> &KdTree<f64, 2> {
        &self.kd_tree
    }

    fn read_positions(&mut self) {
        let reader = self.get_batch_reader();
        for record_batch in reader {
            let record_batch: RecordBatch = match record_batch {
                Ok(batch) => batch,
                Err(e) => panic!("Error reading record batch: {}", e),
            };

            let batch_size = record_batch.num_rows();
            let time_steps: Vec<TimeMS> = read_u64_column(TIME_STEP, &record_batch)
                .into_iter()
                .map(TimeMS::from)
                .collect();
            let agent_ids: Vec<AgentId> = read_u64_column(AGENT_ID, &record_batch)
                .into_iter()
                .map(AgentId::from)
                .collect();
            let x_positions = read_f64_column(COORD_X, &record_batch);
            let y_positions = read_f64_column(COORD_Y, &record_batch);

            for batch in 0..batch_size {
                self.positions
                    .entry(time_steps[batch])
                    .or_default()
                    .push((agent_ids[batch], [x_positions[batch], y_positions[batch]]));
            }
        }
    }

    fn get_batch_reader(&self) -> ParquetRecordBatchReader {
        let map_file = match File::open(&self.file_path) {
            Ok(file) => file,
            Err(e) => panic!("Error reading file from disk: {}", e),
        };
        let builder = match ParquetRecordBatchReaderBuilder::try_new(map_file) {
            Ok(builder) => builder.with_row_groups(vec![self.current_row_group]),
            Err(e) => panic!("Error building parquet reader: {}", e),
        };
        match builder.build() {
            Ok(reader) => reader,
            Err(e) => panic!("Error building reader: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
