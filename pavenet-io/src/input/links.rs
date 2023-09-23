use crate::input::{dfs, files};
use pavenet_config::config::base::{LinkMap, LinkType, MultiLinkMap, TimeStamp, TraceLinkMap};
use polars_core::prelude::DataFrame;
use std::error::Error;
use std::path::PathBuf;

pub enum LinkMapType {
    Single(LinkMap),
    Multiple(MultiLinkMap),
    LinkTraceMap(TraceLinkMap),
}

pub struct LinksReader {
    links_file: PathBuf,
    device_column: String,
    neighbour_column: String,
    is_stream: bool,
    link_type: LinkType,
    streaming_interval: TimeStamp,
}

#[derive(Default)]
pub struct LinksReaderBuilder {
    links_file: PathBuf,
    device_column: String,
    neighbour_column: String,
    is_stream: bool,
    link_type: LinkType,
    streaming_interval: TimeStamp,
}

impl LinksReaderBuilder {
    pub fn new(links_file: PathBuf) -> Self {
        Self::default().links_file(links_file)
    }

    pub fn links_file(mut self, links_file: PathBuf) -> Self {
        self.links_file = links_file;
        self
    }

    pub fn streaming_interval(mut self, streaming_interval: TimeStamp) -> Self {
        self.streaming_interval = streaming_interval;
        self
    }

    pub fn device_column(mut self, device_column: String) -> Self {
        self.device_column = device_column;
        self
    }

    pub fn neighbour_column(mut self, neighbour_column: String) -> Self {
        self.neighbour_column = neighbour_column;
        self
    }

    pub fn stream(mut self, is_stream: bool) -> Self {
        self.is_stream = is_stream;
        self
    }

    pub fn link_type(mut self, link_type: LinkType) -> Self {
        self.link_type = link_type;
        self
    }

    pub fn build(self) -> LinksReader {
        LinksReader {
            links_file: self.links_file,
            device_column: self.device_column,
            neighbour_column: self.neighbour_column,
            is_stream: self.is_stream,
            link_type: self.link_type,
            streaming_interval: self.streaming_interval,
        }
    }
}

impl LinksReader {
    pub(crate) fn builder(trace_file: &PathBuf) -> LinksReaderBuilder {
        LinksReaderBuilder::new(trace_file.clone())
    }

    pub fn links_data(&self, step: TimeStamp) -> Result<LinkMapType, Box<dyn Error>> {
        if self.is_stream != true && self.link_type == LinkType::Single {
            return self.read_single_link_map();
        }
        if self.is_stream != true && self.link_type == LinkType::Multiple {
            return self.read_multiple_link_map();
        }
        return self.stream_multiple_link_map(step);
    }

    fn read_single_link_map(&self) -> Result<LinkMapType, Box<dyn Error>> {
        let links_df = files::read_file(&self.links_file)?;
        let links_map: LinkMap =
            dfs::extract_single_links(&links_df, &self.device_column, &self.neighbour_column)?;
        return Ok(LinkMapType::Single(links_map));
    }

    fn read_multiple_link_map(&self) -> Result<LinkMapType, Box<dyn Error>> {
        let links_df = files::read_file(&self.links_file)?;
        let links_map: MultiLinkMap =
            dfs::extract_multiple_links(&links_df, &self.device_column, &self.neighbour_column)?;
        return Ok(LinkMapType::Multiple(links_map));
    }

    fn stream_multiple_link_map(&self, step: TimeStamp) -> Result<LinkMapType, Box<dyn Error>> {
        let starting_time: TimeStamp = step;
        let ending_time: TimeStamp = step + self.streaming_interval;
        return if self.is_stream == true {
            let links_df: DataFrame =
                files::stream_parquet_in_interval(&self.links_file, starting_time, ending_time)?;
            let static_links: TraceLinkMap =
                dfs::extract_link_traces(&links_df, &self.device_column, &self.neighbour_column)?;
            Ok(LinkMapType::LinkTraceMap(static_links))
        } else {
            let links_df = files::read_file(&self.links_file)?;
            let static_links: TraceLinkMap =
                dfs::extract_link_traces(&links_df, &self.device_column, &self.neighbour_column)?;
            Ok(LinkMapType::LinkTraceMap(static_links))
        };
    }
}
