use crate::batch::{read_u32_column, read_u64_column};
use crate::columns::{NODE_ID, OFF_TIMES, ON_TIMES};
use arrow_array::RecordBatch;
use log::debug;
use parquet::arrow::arrow_reader::{ParquetRecordBatchReader, ParquetRecordBatchReaderBuilder};
use pavenet_engine::bucket::TimeMS;
use pavenet_engine::hashbrown::HashMap;
use pavenet_engine::node::NodeId;
use std::fs::File;
use std::path::PathBuf;

pub type PowerTimes = (Vec<TimeMS>, Vec<TimeMS>);

pub fn read_power_schedule(power_schedule_file: &PathBuf) -> HashMap<NodeId, PowerTimes> {
    let mut power_data_map: HashMap<NodeId, PowerTimes> = HashMap::new();
    let reader = get_batch_reader(power_schedule_file);
    for record_batch in reader {
        let record_batch: RecordBatch = match record_batch {
            Ok(batch) => batch,
            Err(e) => panic!("Error reading record batch: {}", e),
        };

        let node_ids: Vec<NodeId> = read_u64_column(NODE_ID, &record_batch)
            .into_iter()
            .map(NodeId::from)
            .collect();
        let on_times: Vec<TimeMS> = read_u64_column(ON_TIMES, &record_batch)
            .into_iter()
            .map(TimeMS::from)
            .collect();
        let off_times: Vec<TimeMS> = read_u64_column(OFF_TIMES, &record_batch)
            .into_iter()
            .map(TimeMS::from)
            .collect();

        for (idx, node_id) in node_ids.into_iter().enumerate() {
            power_data_map
                .entry(node_id)
                .or_insert((Vec::new(), Vec::new()))
                .0
                .push(on_times[idx]);
            power_data_map
                .entry(node_id)
                .or_insert((Vec::new(), Vec::new()))
                .1
                .push(off_times[idx]);
        }
    }
    power_data_map
}

pub(crate) fn get_batch_reader(file_path: &PathBuf) -> ParquetRecordBatchReader {
    let map_file = match File::open(file_path) {
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
