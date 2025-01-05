use std::fs::File;
use std::path::PathBuf;

use arrow::array::RecordBatch;
use hashbrown::HashMap;
use kiddo::KdTree;
use log::debug;
use parquet::arrow::arrow_reader::{ParquetRecordBatchReader, ParquetRecordBatchReaderBuilder};
use parquet::file::reader::{FileReader, SerializedFileReader};
use serde::Deserialize;

use disolv_core::agent::AgentId;
use disolv_core::bucket::TimeMS;
use disolv_input::batch::{read_f64_column, read_u64_column};
use disolv_input::columns::{AGENT_ID, COORD_X, COORD_Y, TIME_STEP};

use crate::simulation::config::PositionFiles;

pub type AgentIdPos = Vec<(AgentId, [f64; 2])>;
pub type PositionMap = HashMap<TimeMS, AgentIdPos>;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize)]
pub(crate) enum TraceType {
    Constant = 0,
    Mobile,
}

pub(crate) enum Reader {
    Constant(ConstantReader),
    Mobile(MobileReader),
}

impl Reader {
    pub(crate) fn new(position_files: &PositionFiles) -> Self {
        match position_files.trace_type {
            TraceType::Constant => Reader::Constant(ConstantReader::new(position_files)),
            TraceType::Mobile => Reader::Mobile(MobileReader::new(position_files)),
        }
    }

    pub(crate) fn trace_type(&self) -> TraceType {
        match self {
            Reader::Constant(_) => TraceType::Constant,
            Reader::Mobile(_) => TraceType::Mobile,
        }
    }

    pub(crate) fn initialize(&mut self) {
        match self {
            Reader::Constant(reader) => reader.initialize(),
            Reader::Mobile(reader) => reader.initialize(),
        }
    }

    pub(crate) fn update_positions_at(&mut self, time_ms: TimeMS) {
        match self {
            Reader::Constant(_) => {}
            Reader::Mobile(reader) => {
                reader.read_positions_at(time_ms);
                reader.build_kd_tree_at(time_ms);
            }
        }
    }

    pub(crate) fn read_positions_at(&self, time_ms: TimeMS) -> Option<&AgentIdPos> {
        match self {
            Reader::Constant(reader) => Some(&reader.positions),
            Reader::Mobile(reader) => reader.positions.get(&time_ms),
        }
    }

    pub(crate) fn get_kd_tree(&self) -> &KdTree<f64, 2> {
        match self {
            Reader::Constant(reader) => &reader.kd_tree,
            Reader::Mobile(reader) => &reader.kd_tree,
        }
    }
}

#[derive(Debug)]
pub(crate) struct MobileReader {
    pub(crate) file_path: PathBuf,
    pub(crate) current_row_group: usize,
    pub(crate) max_ts_in_row_group: TimeMS,
    pub(crate) positions: PositionMap,
    pub(crate) file_read: bool,
    pub(crate) max_row_groups: usize,
    pub(crate) kd_tree: KdTree<f64, 2>,
}

impl MobileReader {
    fn new(position_files: &PositionFiles) -> Self {
        Self {
            file_path: PathBuf::from(position_files.position_file.to_owned()),
            positions: PositionMap::default(),
            current_row_group: usize::default(),
            kd_tree: KdTree::default(),
            file_read: false,
            max_row_groups: 0,
            max_ts_in_row_group: TimeMS::default(),
        }
    }

    fn initialize(&mut self) {
        if let Ok(file) = File::open(&self.file_path) {
            let reader = SerializedFileReader::new(file).expect("Failed to read file");
            let parquet_metadata = reader.metadata();
            self.max_row_groups = parquet_metadata.num_row_groups();
        }
    }

    fn read_positions_at(&mut self, time_ms: TimeMS) {
        if (self.positions.contains_key(&time_ms) && self.max_ts_in_row_group != time_ms)
            || self.file_read
        {
            return;
        }
        debug!("Reading positions at {}", time_ms);
        self.read_positions();
        self.current_row_group += 1;
        if self.current_row_group == self.max_row_groups {
            self.file_read = true;
        }
    }

    fn build_kd_tree_at(&mut self, time_ms: TimeMS) {
        self.kd_tree = KdTree::default();
        if let Some(positions) = self.positions.get(&time_ms) {
            positions.iter().for_each(|pos| {
                self.kd_tree.add(&pos.1, pos.0.as_u64());
            });
        };
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
            self.max_ts_in_row_group = *time_steps.iter().max().expect("cannot find max time");
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

#[derive(Debug)]
pub(crate) struct ConstantReader {
    pub(crate) file_path: PathBuf,
    pub(crate) positions: AgentIdPos,
    pub(crate) kd_tree: KdTree<f64, 2>,
}

impl ConstantReader {
    fn new(position_files: &PositionFiles) -> Self {
        Self {
            file_path: PathBuf::from(position_files.position_file.to_owned()),
            positions: AgentIdPos::default(),
            kd_tree: KdTree::default(),
        }
    }

    fn initialize(&mut self) {
        self.read_all_positions();
        self.build_kd_tree();
    }

    fn build_kd_tree(&mut self) {
        self.positions.iter().for_each(|pos| {
            self.kd_tree.add(&pos.1, pos.0.as_u64());
        });
    }

    fn read_all_positions(&mut self) {
        let reader = self.get_batch_reader();
        for record_batch in reader {
            let record_batch: RecordBatch = match record_batch {
                Ok(batch) => batch,
                Err(e) => panic!("Error reading record batch: {}", e),
            };

            let batch_size = record_batch.num_rows();
            let agent_ids: Vec<AgentId> = read_u64_column(AGENT_ID, &record_batch)
                .into_iter()
                .map(AgentId::from)
                .collect();
            let x_positions = read_f64_column(COORD_X, &record_batch);
            let y_positions = read_f64_column(COORD_Y, &record_batch);

            for batch in 0..batch_size {
                self.positions
                    .push((agent_ids[batch], [x_positions[batch], y_positions[batch]]));
            }
        }
    }

    fn get_batch_reader(&self) -> ParquetRecordBatchReader {
        debug!("Reading file {}", &self.file_path.display());
        let map_file = match File::open(&self.file_path) {
            Ok(file) => file,
            Err(e) => panic!("Error reading file from disk: {}", e),
        };
        let builder = match ParquetRecordBatchReaderBuilder::try_new(map_file) {
            Ok(builder) => builder,
            Err(e) => panic!("Error building parquet reader: {}", e),
        };
        match builder.build() {
            Ok(reader) => reader,
            Err(e) => panic!("Error building reader: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {}
