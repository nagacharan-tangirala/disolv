pub mod data {
    use crate::file_reader::{read_file, stream_parquet_in_interval};
    use crate::links::df::extract_link_traces;
    use pavenet_core::link::DLink;
    use pavenet_engine::bucket::TimeS;
    use pavenet_engine::entity::NodeId;
    use pavenet_engine::hashbrown::HashMap;
    use std::path::PathBuf;
    use typed_builder::TypedBuilder;

    pub type LinkMap = HashMap<TimeS, HashMap<NodeId, Vec<DLink>>>;

    #[derive(Clone)]
    pub enum LinkReader {
        File(ReadLinks),
        Stream(StreamLinks),
    }

    impl LinkReader {
        pub fn new(
            links_file: PathBuf,
            streaming_interval: TimeS,
            is_streaming: bool,
        ) -> LinkReader {
            if is_streaming {
                LinkReader::Stream(
                    StreamLinks::builder()
                        .links_file(links_file)
                        .streaming_interval(streaming_interval)
                        .build(),
                )
            } else {
                LinkReader::File(ReadLinks::builder().links_file(links_file).build())
            }
        }
    }

    #[derive(Clone, TypedBuilder)]
    pub struct ReadLinks {
        links_file: PathBuf,
    }

    impl ReadLinks {
        pub fn read_links_data(&self, _step: TimeS) -> LinkMap {
            let links_df = match read_file(&self.links_file) {
                Ok(links_df) => links_df,
                Err(e) => panic!("ReadLinks::read_links_data error {:?}", e),
            };
            match extract_link_traces(&links_df) {
                Ok(links) => links,
                Err(e) => panic!("ReadLinks::read_links_data error {:?}", e),
            }
        }
    }

    #[derive(Clone, TypedBuilder)]
    pub struct StreamLinks {
        links_file: PathBuf,
        streaming_interval: TimeS,
    }

    impl StreamLinks {
        pub fn stream_links_data(&self, step: TimeS) -> LinkMap {
            let starting_time = step;
            let ending_time = step + self.streaming_interval;
            let links_df =
                match stream_parquet_in_interval(&self.links_file, starting_time, ending_time) {
                    Ok(links_df) => links_df,
                    Err(e) => panic!("StreamLinks::fetch_links_data error {:?}", e),
                };
            match extract_link_traces(&links_df) {
                Ok(links) => links,
                Err(e) => panic!("ReadLinks::read_links_data error {:?}", e),
            }
        }
    }
}

pub(super) mod df {
    use crate::columns::{DISTANCE, LOAD_FACTOR, NODE_ID, TARGET_ID, TIME_STEP};
    use crate::converter::series::{to_f32_vec, to_nodeid_vec, to_timestamp_vec};
    use crate::links::data::LinkMap;
    use log::debug;
    use pavenet_core::link::DLink;
    use pavenet_engine::bucket::TimeS;
    use pavenet_engine::entity::NodeId;
    use pavenet_engine::hashbrown::HashMap;
    use polars::error::ErrString;
    use polars::prelude::{col, lit, DataFrame, IntoLazy, PolarsError, PolarsResult, Series};

    mod mandatory {
        use crate::columns::*;

        pub const COLUMNS: [&str; 2] = [TIME_STEP, TARGET_ID];
    }

    mod optional {
        use crate::columns::*;

        pub const COLUMNS: [&str; 2] = [DISTANCE, LOAD_FACTOR];
    }

    pub(crate) fn extract_link_traces(
        links_df: &DataFrame,
    ) -> Result<LinkMap, Box<dyn std::error::Error>> {
        validate_links_df(links_df)?;
        let ts_series: &Series = links_df.column(TIME_STEP)?;
        let time_stamps: Vec<TimeS> = to_timestamp_vec(ts_series)?;
        let mut links: LinkMap = HashMap::with_capacity(time_stamps.len());

        for time_stamp in time_stamps.iter() {
            let ts_df = links_df
                .clone()
                .lazy()
                .filter(col(TIME_STEP).eq(lit(time_stamp.as_u64())))
                .collect()?;

            if ts_df.height() == 0 {
                links.entry(*time_stamp).or_insert(HashMap::new());
                continue;
            }

            let id_series: Series = ts_df.column(NODE_ID)?.explode()?;
            let node_ids: Vec<NodeId> = to_nodeid_vec(&id_series)?;
            let link_vec: Vec<DLink> = build_links_data(&ts_df)?;

            let mut link_map_entry: HashMap<NodeId, Vec<DLink>> =
                HashMap::with_capacity(node_ids.len());
            for (node_id, link) in node_ids.into_iter().zip(link_vec.into_iter()) {
                link_map_entry
                    .entry(node_id)
                    .or_insert(Vec::new())
                    .push(link);
            }
            links.entry(*time_stamp).or_insert(link_map_entry);
        }
        return Ok(links);
    }

    fn validate_links_df(df: &DataFrame) -> Result<(), PolarsError> {
        for column in mandatory::COLUMNS.iter() {
            if !df.get_column_names().contains(column) {
                return Err(PolarsError::ColumnNotFound(ErrString::from(
                    column.to_string(),
                )));
            }
        }
        return Ok(());
    }

    fn build_links_data(df: &DataFrame) -> Result<Vec<DLink>, Box<dyn std::error::Error>> {
        let target_id_series: &Series = df.column(TARGET_ID)?;
        let target_ids: Vec<NodeId> = to_nodeid_vec(&target_id_series)?;
        let mut link_vec: Vec<DLink> = target_ids
            .into_iter()
            .map(|target_id_vec| DLink::new(target_id_vec))
            .collect();

        let optional_columns = get_optional_columns(df);
        for optional_col in optional_columns.into_iter() {
            match optional_col {
                DISTANCE => {
                    let distance_series: &Series = df.column(DISTANCE)?;
                    let distances: Vec<f32> = to_f32_vec(&distance_series)?;
                    for (idx, distance) in distances.into_iter().enumerate() {
                        link_vec[idx].properties.distance = Some(distance);
                    }
                }
                LOAD_FACTOR => {
                    let lf_series: &Series = df.column(LOAD_FACTOR)?;
                    let load_factors: Vec<f32> = to_f32_vec(&lf_series)?;
                    for (idx, load_factor) in load_factors.into_iter().enumerate() {
                        link_vec[idx].properties.load_factor = Some(load_factor);
                    }
                }
                _ => return Err("Invalid column name".into()),
            }
        }
        return Ok(link_vec);
    }

    fn get_optional_columns(df: &DataFrame) -> Vec<&str> {
        return df
            .get_column_names()
            .into_iter()
            .filter(|col| optional::COLUMNS.contains(&col))
            .map(|col| col)
            .collect();
    }
}
