use crate::dfs::links;
use crate::input::files;
use hashbrown::HashMap;
use pavenet_core::structs::Link;
use pavenet_core::types::{NodeId, TimeStamp};
use polars_core::prelude::DataFrame;
use std::error::Error;
use std::path::PathBuf;
use typed_builder::TypedBuilder;

pub type LinkMap = HashMap<TimeStamp, HashMap<NodeId, Link>>;

pub enum LinkReaderType {
    File(ReadLinks),
    Stream(StreamLinks),
}

pub trait LinksFetcher {
    fn fetch_links_data(&self, step: TimeStamp) -> Result<LinkMap, Box<dyn Error>>;
}

#[derive(TypedBuilder)]
pub struct ReadLinks {
    links_file: PathBuf,
}

impl LinksFetcher for ReadLinks {
    fn fetch_links_data(&self, _step: TimeStamp) -> Result<LinkMap, Box<dyn Error>> {
        let links_df = files::read_file(&self.links_file)?;
        links::extract_link_traces(&links_df)
    }
}

#[derive(TypedBuilder)]
pub struct StreamLinks {
    links_file: PathBuf,
    streaming_interval: TimeStamp,
}

impl LinksFetcher for StreamLinks {
    fn fetch_links_data(&self, step: TimeStamp) -> Result<LinkMap, Box<dyn Error>> {
        let starting_time = step;
        let ending_time = step + self.streaming_interval;
        let links_df: DataFrame =
            files::stream_parquet_in_interval(&self.links_file, starting_time, ending_time)?;
        links::extract_link_traces(&links_df)
    }
}
