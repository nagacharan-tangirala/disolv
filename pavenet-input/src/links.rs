pub mod data {
    use crate::file_reader::{read_file, stream_parquet_in_interval};
    use crate::links::df::extract_link_traces;
    use hashbrown::HashMap;
    use pavenet_recipe::link::Link;
    use pavenet_recipe::node_info::id::NodeId;
    use pavenet_recipe::times::ts::TimeS;
    use polars::prelude::DataFrame;
    use std::error::Error;
    use std::path::PathBuf;
    use typed_builder::TypedBuilder;

    pub type LinkMap = HashMap<TimeS, HashMap<NodeId, Link>>;

    #[derive(Clone)]
    pub enum LinkReader {
        File(ReadLinks),
        Stream(StreamLinks),
    }

    pub trait LinksFetcher {
        fn fetch_links_data(&self, step: TimeS) -> Result<LinkMap, Box<dyn Error>>;
    }

    #[derive(Clone, TypedBuilder)]
    pub struct ReadLinks {
        links_file: PathBuf,
    }

    impl LinksFetcher for ReadLinks {
        fn fetch_links_data(&self, _step: TimeS) -> Result<LinkMap, Box<dyn Error>> {
            let links_df = read_file(&self.links_file)?;
            extract_link_traces(&links_df)
        }
    }

    #[derive(Clone, TypedBuilder)]
    pub struct StreamLinks {
        links_file: PathBuf,
        streaming_interval: TimeS,
    }

    impl LinksFetcher for StreamLinks {
        fn fetch_links_data(&self, step: TimeS) -> Result<LinkMap, Box<dyn Error>> {
            let starting_time = step;
            let ending_time = step + self.streaming_interval;
            let links_df: DataFrame =
                stream_parquet_in_interval(&self.links_file, starting_time, ending_time)?;
            extract_link_traces(&links_df)
        }
    }
}

pub(super) mod df {
    use crate::columns::{DISTANCE, LOAD_FACTOR, NODE_ID, TARGET_ID, TIME_STEP};
    use crate::converter::list_series::{to_vec_of_f32_vec, to_vec_of_nodeid_vec};
    use crate::converter::series::{to_nodeid_vec, to_timestamp_vec};
    use crate::links::data::LinkMap;
    use hashbrown::HashMap;
    use pavenet_recipe::link::Link;
    use pavenet_recipe::node_info::id::NodeId;
    use pavenet_recipe::times::ts::TimeS;
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
        let filtered_df = filter_links_df(links_df)?;

        let ts_series: &Series = filtered_df.column(TIME_STEP)?;
        let time_stamps: Vec<TimeS> = to_timestamp_vec(ts_series)?;
        let mut links: LinkMap = HashMap::with_capacity(time_stamps.len());

        for time_stamp in time_stamps.iter() {
            let ts_df = filtered_df
                .clone()
                .lazy()
                .filter(col(TIME_STEP).eq(lit(time_stamp.as_u64())))
                .collect()?;

            if ts_df.height() == 0 {
                links.entry(*time_stamp).or_insert(HashMap::new());
                continue;
            }

            let id_series: &Series = ts_df.column(NODE_ID)?;
            let node_ids: Vec<NodeId> = to_nodeid_vec(id_series)?;

            let mut link_vec: Vec<Link> = extract_mandatory_data(&ts_df)?;
            add_optional_data(&ts_df, &mut link_vec)?;

            let mut link_map: HashMap<NodeId, Link> = HashMap::with_capacity(node_ids.len());
            for (idx, node_link) in link_vec.into_iter().enumerate() {
                link_map.insert(node_ids[idx], node_link);
            }
            links.entry(*time_stamp).or_insert(link_map);
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

    fn get_columns_to_group_by(links_df: &DataFrame) -> Vec<polars::prelude::Expr> {
        let mut columns = links_df.get_column_names();
        columns.remove(columns.iter().position(|x| *x == TIME_STEP).unwrap());
        let column_vec = columns
            .iter()
            .map(|x| col(x))
            .collect::<Vec<polars::prelude::Expr>>();
        return column_vec;
    }

    fn filter_links_df(links_df: &DataFrame) -> PolarsResult<DataFrame> {
        let column_vec = get_columns_to_group_by(links_df);
        let filtered_df: DataFrame = links_df
            .clone()
            .lazy()
            .group_by([col(TIME_STEP)])
            .agg(column_vec.into_iter().collect::<Vec<_>>())
            .collect()?;
        return Ok(filtered_df);
    }

    fn extract_mandatory_data(df: &DataFrame) -> Result<Vec<Link>, Box<dyn std::error::Error>> {
        let target_id_series: &Series = df.column(TARGET_ID)?;
        let target_ids: Vec<Vec<NodeId>> = to_vec_of_nodeid_vec(&target_id_series)?;

        let links: Vec<Link> = target_ids
            .into_iter()
            .map(|target_id| Link::new())
            .collect();
        return Ok(links);
    }

    fn add_optional_data(
        df: &DataFrame,
        links: &mut Vec<Link>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let optional_columns = get_optional_columns(df);
        for optional_col in optional_columns.into_iter() {
            match optional_col {
                DISTANCE => {
                    let distance_series: &Series = df.column(DISTANCE)?;
                    let distances: Vec<Vec<f32>> = to_vec_of_f32_vec(&distance_series)?;
                    for (idx, distance) in distances.into_iter().enumerate() {
                        links[idx].distance = Some(distance);
                    }
                }
                LOAD_FACTOR => {
                    let lf_series: &Series = df.column(LOAD_FACTOR)?;
                    let load_factors: Vec<Vec<f32>> = to_vec_of_f32_vec(&lf_series)?;
                    for (idx, load_factor) in load_factors.into_iter().enumerate() {
                        links[idx].load_factor = Some(load_factor);
                    }
                }
                _ => return Err("Invalid column name".into()),
            }
        }
        return Ok(());
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
