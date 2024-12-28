use std::fs::File;
use std::path::PathBuf;

use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_reader::{ParquetRecordBatchReader, ParquetRecordBatchReaderBuilder};

use disolv_core::agent::AgentId;
use disolv_core::bucket::TimeMS;
use hashbrown::HashMap;

use crate::batch::read_u64_column;
use crate::columns::{AGENT_ID, OFF_TIMES, ON_TIMES};

pub type PowerTimes = (Vec<TimeMS>, Vec<TimeMS>);

pub fn read_power_schedule(power_schedule_file: &PathBuf) -> HashMap<AgentId, PowerTimes> {
    let mut power_data_map: HashMap<AgentId, PowerTimes> = HashMap::new();
    let reader = get_batch_reader(power_schedule_file);
    for record_batch in reader {
        let record_batch: RecordBatch = match record_batch {
            Ok(batch) => batch,
            Err(e) => panic!("Error reading record batch: {}", e),
        };

        let agent_ids: Vec<AgentId> = read_u64_column(AGENT_ID, &record_batch)
            .into_iter()
            .map(AgentId::from)
            .collect();
        let on_times: Vec<TimeMS> = read_u64_column(ON_TIMES, &record_batch)
            .into_iter()
            .map(TimeMS::from)
            .collect();
        let off_times: Vec<TimeMS> = read_u64_column(OFF_TIMES, &record_batch)
            .into_iter()
            .map(TimeMS::from)
            .collect();

        for (idx, agent_id) in agent_ids.into_iter().enumerate() {
            power_data_map
                .entry(agent_id)
                .or_insert((Vec::new(), Vec::new()))
                .0
                .push(on_times[idx]);
            power_data_map
                .entry(agent_id)
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
