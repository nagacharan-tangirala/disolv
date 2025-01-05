use std::fs::File;
use std::path::PathBuf;

use arrow::record_batch::RecordBatch;
use hashbrown::HashMap;
use log::debug;
use parquet::arrow::arrow_reader::{ParquetRecordBatchReader, ParquetRecordBatchReaderBuilder};
use typed_builder::TypedBuilder;

use disolv_core::agent::AgentId;
use disolv_core::bucket::TimeMS;
use disolv_models::device::mobility::road::RoadId;
use disolv_models::device::mobility::velocity::Velocity;
use disolv_models::device::mobility::{MapState, Point2D};

use crate::batch::{get_row_groups_for_time, read_f64_column, read_u32_column, read_u64_column};
use crate::columns::{AGENT_ID, COORD_X, COORD_Y, COORD_Z, ROAD_ID, TIME_STEP, VELOCITY};

pub type TraceMap = HashMap<TimeMS, HashMap<AgentId, MapState>>;

#[derive(Clone, Debug, TypedBuilder)]
pub struct MapReader {
    pub is_streaming: bool,
    file_path: PathBuf,
    streaming_step: TimeMS,
}

impl MapReader {
    pub fn fetch_traffic_data(&self, step: TimeMS) -> TraceMap {
        let mut trace_map: TraceMap = HashMap::new();
        let reader = self.get_batch_reader(step);
        debug!("Reading map data for step: {}", step);

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

            let z_positions = if record_batch.column_by_name(COORD_Z).is_some() {
                read_f64_column(COORD_Z, &record_batch)
            } else {
                Vec::new()
            };

            let velocity = if record_batch.column_by_name(VELOCITY).is_some() {
                read_f64_column(VELOCITY, &record_batch)
                    .into_iter()
                    .map(Velocity::from)
                    .collect()
            } else {
                Vec::new()
            };

            let road_ids = if record_batch.column_by_name(ROAD_ID).is_some() {
                read_u32_column(ROAD_ID, &record_batch)
                    .into_iter()
                    .map(RoadId::from)
                    .collect()
            } else {
                Vec::new()
            };

            for i in 0..batch_size {
                let mut map_state = MapState::builder()
                    .pos(Point2D {
                        x: x_positions[i],
                        y: y_positions[i],
                    })
                    .build();
                if !road_ids.is_empty() {
                    map_state.road_id = Some(road_ids[i]);
                }
                if !z_positions.is_empty() {
                    map_state.z = Some(z_positions[i]);
                }
                if !velocity.is_empty() {
                    map_state.velocity = Some(velocity[i]);
                }

                trace_map
                    .entry(time_steps[i])
                    .or_default()
                    .insert(agent_ids[i], map_state);
            }
        }
        trace_map
    }

    pub(crate) fn get_batch_reader(&self, step: TimeMS) -> ParquetRecordBatchReader {
        let start_interval = step;
        let end_interval = step + self.streaming_step;
        let map_file = match File::open(&self.file_path) {
            Ok(file) => file,
            Err(e) => panic!("Error reading file from disk: {}", e),
        };
        let selected_groups = get_row_groups_for_time(
            &self.file_path,
            self.is_streaming,
            start_interval,
            end_interval,
        );
        let builder = match ParquetRecordBatchReaderBuilder::try_new(map_file) {
            Ok(builder) => builder.with_row_groups(selected_groups),
            Err(e) => panic!("Error building parquet reader: {}", e),
        };
        match builder.build() {
            Ok(reader) => reader,
            Err(e) => panic!("Error building reader: {}", e),
        }
    }
}
