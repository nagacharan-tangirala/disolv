use std::fs::File;
use std::path::PathBuf;

use log::debug;
use parquet::arrow::arrow_reader::{ParquetRecordBatchReader, ParquetRecordBatchReaderBuilder};
use typed_builder::TypedBuilder;

use disolv_core::agent::AgentId;
use disolv_core::bucket::TimeMS;
use disolv_core::hashbrown::HashMap;
use disolv_core::radio::Link;
use disolv_models::net::radio::LinkProperties;

use crate::batch::{get_row_groups_for_time, read_f64_column, read_u64_column};
use crate::columns::{AGENT_ID, DISTANCE, LOAD_FACTOR, TARGET_ID, TIME_STEP};

pub type LinkMap = HashMap<TimeMS, HashMap<AgentId, Vec<Link<LinkProperties>>>>;

#[derive(Clone, TypedBuilder)]
pub struct LinkReader {
    pub is_streaming: bool,
    pub file_path: PathBuf,
    streaming_step: TimeMS,
}

impl LinkReader {
    pub fn fetch_links_data(&self, step: TimeMS) -> LinkMap {
        let mut link_map: LinkMap = HashMap::new();
        let reader = self.get_batch_reader(step);

        for record_bath in reader {
            let record_batch = match record_bath {
                Ok(batch) => batch,
                Err(e) => panic!("Error reading record batch: {}", e),
            };
            let time_steps: Vec<TimeMS> = read_u64_column(TIME_STEP, &record_batch)
                .into_iter()
                .map(TimeMS::from)
                .collect();
            let agent_ids: Vec<AgentId> = read_u64_column(AGENT_ID, &record_batch)
                .into_iter()
                .map(AgentId::from)
                .collect();
            let target_ids: Vec<AgentId> = read_u64_column(TARGET_ID, &record_batch)
                .into_iter()
                .map(AgentId::from)
                .collect();
            let mut link_vec: Vec<Link<LinkProperties>> =
                target_ids.into_iter().map(Link::new).collect();

            let distance: Vec<f64>;
            if record_batch.column_by_name(DISTANCE).is_some() {
                distance = read_f64_column(DISTANCE, &record_batch);
                for (idx, link) in link_vec.iter_mut().enumerate() {
                    link.properties.distance = Some(distance[idx] as f32);
                }
            }

            let load_factor: Vec<f64>;
            if record_batch.column_by_name(LOAD_FACTOR).is_some() {
                load_factor = read_f64_column(LOAD_FACTOR, &record_batch);
                for (idx, link) in link_vec.iter_mut().enumerate() {
                    link.properties.load_factor = Some(load_factor[idx] as f32);
                }
            }

            for ((time, agent_id), link) in time_steps
                .into_iter()
                .zip(agent_ids.into_iter())
                .zip(link_vec.into_iter())
            {
                link_map
                    .entry(time)
                    .or_default()
                    .entry(agent_id)
                    .or_default()
                    .push(link);
            }
        }
        link_map
    }

    pub(crate) fn get_batch_reader(&self, step: TimeMS) -> ParquetRecordBatchReader {
        let start_interval = step;
        let end_interval = step + self.streaming_step;
        let map_file = match File::open(&self.file_path) {
            Ok(file) => file,
            Err(e) => {
                debug!("Error reading file from disk: {}", e);
                panic!("Error reading file from disk: {}", e);
            }
        };

        let selected_groups = get_row_groups_for_time(
            &self.file_path,
            self.is_streaming,
            start_interval,
            end_interval,
        );
        let builder = match ParquetRecordBatchReaderBuilder::try_new(map_file) {
            Ok(builder) => builder.with_row_groups(selected_groups),
            Err(e) => {
                debug!("Error building parquet reader: {}", e);
                panic!("Error building parquet reader: {}", e);
            }
        };
        match builder.build() {
            Ok(reader) => reader,
            Err(e) => {
                debug!("Error building reader: {}", e);
                panic!("Error building reader: {}", e);
            }
        }
    }
}
