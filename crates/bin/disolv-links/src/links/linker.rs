use std::fs::File;
use std::sync::Arc;

use arrow::array::{ArrayRef, Float64Array, RecordBatch, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use kiddo::{KdTree, NearestNeighbour, SquaredEuclidean};
use log::debug;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;
use serde::Deserialize;

use disolv_core::agent::AgentId;
use disolv_core::bucket::TimeMS;
use disolv_input::columns::{AGENT_ID, DISTANCE, TARGET_ID, TIME_STEP};

use crate::links::reader::AgentIdPos;
use crate::simulation::config::LinkSettings;

#[derive(Copy, Clone, Default, Debug, Deserialize)]
pub struct Radius(f64);

impl From<f64> for Radius {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl Radius {
    pub(crate) fn as_f64(&self) -> f64 {
        self.0
    }
}

#[derive(Copy, Clone, Default, Debug, Deserialize)]
pub struct DeviceCount(u32);

impl From<u32> for DeviceCount {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl DeviceCount {
    pub(crate) fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) enum LinkType {
    Static,
    Dynamic,
}

#[derive(Clone, Default)]
struct WriterCache {
    cache_size: usize,
    sources: Vec<u64>,
    targets: Vec<u64>,
    distances: Vec<f64>,
    times: Vec<u64>,
}

impl WriterCache {
    fn new(cache_size: usize) -> Self {
        Self {
            sources: Vec::with_capacity(cache_size),
            targets: Vec::with_capacity(cache_size),
            distances: Vec::with_capacity(cache_size),
            times: Vec::with_capacity(cache_size),
            cache_size: cache_size,
        }
    }

    fn is_full(&self) -> bool {
        self.sources.len() > self.cache_size
    }

    fn as_record_batch(&mut self) -> RecordBatch {
        RecordBatch::try_from_iter(vec![
            (
                TIME_STEP,
                Arc::new(UInt64Array::from(std::mem::take(&mut self.times))) as ArrayRef,
            ),
            (
                AGENT_ID,
                Arc::new(UInt64Array::from(std::mem::take(&mut self.sources))) as ArrayRef,
            ),
            (
                TARGET_ID,
                Arc::new(UInt64Array::from(std::mem::take(&mut self.targets))) as ArrayRef,
            ),
            (
                DISTANCE,
                Arc::new(Float64Array::from(std::mem::take(&mut self.distances))) as ArrayRef,
            ),
        ])
        .expect("Failed to convert writer cache to record batch")
    }
}

pub(crate) enum LinkerImpl {
    Circular(CircularLinker),
    Unicast(UnicastLinker),
}

impl LinkerImpl {
    pub(crate) fn new(output_path: &str, link_settings: &LinkSettings) -> LinkerImpl {
        let output_file = output_path.to_owned() + link_settings.links_file.as_str() + ".parquet";
        let writer = Self::create_writer(&output_file);
        match link_settings.link_model.to_lowercase().as_str() {
            "circular" => LinkerImpl::Circular(CircularLinker::new(writer, link_settings)),
            "unicast" => LinkerImpl::Unicast(UnicastLinker::new(writer, link_settings)),
            _ => panic!("Invalid linker model"),
        }
    }

    pub(crate) fn write_links(
        &mut self,
        source_positions: &AgentIdPos,
        target_tree: &KdTree<f64, 2>,
        now: TimeMS,
    ) {
        match self {
            LinkerImpl::Circular(circular) => {
                circular.write_links(source_positions, target_tree, now)
            }
            LinkerImpl::Unicast(unicast) => unicast.write_links(source_positions, target_tree, now),
        }
    }

    fn create_writer(output_file: &str) -> ArrowWriter<File> {
        debug!("Creating links file {}", output_file);
        let time_ms = Field::new(TIME_STEP, DataType::UInt64, false);
        let source_id = Field::new(AGENT_ID, DataType::UInt64, false);
        let target_id = Field::new(TARGET_ID, DataType::UInt64, false);
        let distance = Field::new(DISTANCE, DataType::Float64, false);

        let props = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .build();

        let schema = Schema::new(vec![time_ms, source_id, target_id, distance]);
        let output_file = match File::create(output_file) {
            Ok(file) => file,
            Err(_) => panic!("Failed to create links file to write"),
        };
        match ArrowWriter::try_new(output_file, SchemaRef::from(schema), Some(props)) {
            Ok(writer) => writer,
            Err(_) => panic!("Failed to create links file writer"),
        }
    }

    pub(crate) fn flush(mut self) {
        match self {
            LinkerImpl::Circular(circular) => circular.flush(),
            LinkerImpl::Unicast(unicast) => unicast.flush(),
        }
    }
}

pub(crate) struct CircularLinker {
    writer: ArrowWriter<File>,
    linker_settings: LinkSettings,
    writer_cache: WriterCache,
}

impl CircularLinker {
    fn new(writer: ArrowWriter<File>, link_settings: &LinkSettings) -> Self {
        Self {
            writer: writer,
            linker_settings: link_settings.clone(),
            writer_cache: WriterCache::new(125000),
        }
    }

    pub(crate) fn write_links(
        &mut self,
        source_positions: &AgentIdPos,
        target_tree: &KdTree<f64, 2>,
        now: TimeMS,
    ) {
        debug!("Calculating links for {}", now);

        for agent_id_pos in source_positions.iter() {
            if let Some(radius) = self.linker_settings.link_radius {
                let neighbours: Vec<NearestNeighbour<f64, u64>> = target_tree
                    .within::<SquaredEuclidean>(&agent_id_pos.1, radius.as_f64() * radius.as_f64());

                neighbours.into_iter().for_each(|neigh_dist| {
                    if neigh_dist.distance > 0. {
                        self.writer_cache.times.push(now.as_u64());
                        self.writer_cache.sources.push(agent_id_pos.0.as_u64());
                        self.writer_cache.targets.push(neigh_dist.item);
                        self.writer_cache.distances.push(neigh_dist.distance.sqrt());
                    }
                });
                continue;
            }

            if let Some(count) = self.linker_settings.link_count {
                let neighbours =
                    target_tree.nearest_n::<SquaredEuclidean>(&agent_id_pos.1, count.as_usize());

                neighbours.into_iter().for_each(|neigh_dist| {
                    if neigh_dist.distance > 0. {
                        self.writer_cache.times.push(now.as_u64());
                        self.writer_cache.sources.push(agent_id_pos.0.as_u64());
                        self.writer_cache.targets.push(neigh_dist.item);
                        self.writer_cache.distances.push(neigh_dist.distance.sqrt());
                    }
                });
            }
        }
        if self.writer_cache.is_full() {
            debug!("Cache full. Writing to files");
            self.writer
                .write(&self.writer_cache.as_record_batch())
                .expect("Failed to write record batches to file");
        }
    }

    pub(crate) fn flush(mut self) {
        debug!(r"Link calculation done. Flushing the cache to file");
        self.writer
            .write(&self.writer_cache.as_record_batch())
            .expect("Failed to flush link cache");
        self.writer.close().expect("Failed to close the link file");
    }
}

pub struct UnicastLinker {
    writer: ArrowWriter<File>,
    linker_settings: LinkSettings,
    writer_cache: WriterCache,
}

impl UnicastLinker {
    fn new(writer: ArrowWriter<File>, link_settings: &LinkSettings) -> Self {
        Self {
            writer: writer,
            linker_settings: link_settings.clone(),
            writer_cache: WriterCache::new(125000),
        }
    }

    pub(crate) fn write_links(
        &mut self,
        source_positions: &AgentIdPos,
        target_tree: &KdTree<f64, 2>,
        now: TimeMS,
    ) {
        debug!("Calculating links for {}", now);

        for agent_id_pos in source_positions.iter() {
            if let Some(radius) = self.linker_settings.link_radius {
                let neighbours: Vec<NearestNeighbour<f64, u64>> = target_tree
                    .within::<SquaredEuclidean>(&agent_id_pos.1, radius.as_f64() * radius.as_f64());

                neighbours.into_iter().for_each(|neigh_dist| {
                    if neigh_dist.distance > 0. {
                        self.writer_cache.times.push(now.as_u64());
                        self.writer_cache.sources.push(agent_id_pos.0.as_u64());
                        self.writer_cache.targets.push(neigh_dist.item);
                        self.writer_cache.distances.push(neigh_dist.distance.sqrt());
                    }
                });
                continue;
            }

            if let Some(count) = self.linker_settings.link_count {
                let neighbours =
                    target_tree.nearest_n::<SquaredEuclidean>(&agent_id_pos.1, count.as_usize());

                neighbours.into_iter().for_each(|neigh_dist| {
                    if neigh_dist.distance > 0. {
                        self.writer_cache.times.push(now.as_u64());
                        self.writer_cache.sources.push(neigh_dist.item);
                        self.writer_cache.targets.push(agent_id_pos.0.as_u64());
                        self.writer_cache.distances.push(neigh_dist.distance.sqrt());
                    }
                });
            }
        }

        if self.writer_cache.is_full() {
            debug!("Cache full. Writing to files");
            self.writer
                .write(&self.writer_cache.as_record_batch())
                .expect("Failed to write record batches to file");
        }
    }

    fn flush(mut self) {
        debug!(r"Link calculation done. Flushing the cache to file");
        self.writer
            .write(&self.writer_cache.as_record_batch())
            .expect("Failed to flush link cache");
        self.writer.close().expect("Failed to close the link file");
    }
}
