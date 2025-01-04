use std::mem;
use std::path::Path;
use std::sync::Arc;

use arrow::array::{ArrayRef, Float64Array, RecordBatch, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use kiddo::{KdTree, NearestNeighbour, SquaredEuclidean};
use log::debug;
use serde::Deserialize;

use disolv_core::bucket::TimeMS;
use disolv_input::columns::{AGENT_ID, DISTANCE, TARGET_ID, TIME_STEP};
use disolv_output::result::ResultWriter;
use disolv_output::writer::WriterType;

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
    destinations: Vec<u64>,
    distances: Vec<f64>,
    times: Vec<u64>,
}

impl WriterCache {
    fn new(cache_size: usize) -> Self {
        Self {
            sources: Vec::with_capacity(cache_size),
            destinations: Vec::with_capacity(cache_size),
            distances: Vec::with_capacity(cache_size),
            times: Vec::with_capacity(cache_size),
            cache_size,
        }
    }

    fn is_full(&self) -> bool {
        self.sources.len() > self.cache_size
    }

    fn as_record_batch(&mut self) -> RecordBatch {
        RecordBatch::try_from_iter(vec![
            (
                TIME_STEP,
                Arc::new(UInt64Array::from(mem::take(&mut self.times))) as ArrayRef,
            ),
            (
                AGENT_ID,
                Arc::new(UInt64Array::from(mem::take(&mut self.sources))) as ArrayRef,
            ),
            (
                TARGET_ID,
                Arc::new(UInt64Array::from(mem::take(&mut self.destinations))) as ArrayRef,
            ),
            (
                DISTANCE,
                Arc::new(Float64Array::from(mem::take(&mut self.distances))) as ArrayRef,
            ),
        ])
        .expect("Failed to convert writer cache to record batch")
    }
}

pub(crate) enum LinkerImpl {
    Circular(CircularLinker),
    ReverseUnicast(ReverseUnicastLinker),
}

impl ResultWriter for LinkerImpl {
    fn schema() -> Schema {
        let time_ms = Field::new(TIME_STEP, DataType::UInt64, false);
        let source_id = Field::new(AGENT_ID, DataType::UInt64, false);
        let destination_id = Field::new(TARGET_ID, DataType::UInt64, false);
        let distance = Field::new(DISTANCE, DataType::Float64, false);
        Schema::new(vec![time_ms, source_id, destination_id, distance])
    }

    fn write_to_file(&mut self) {
        match self {
            LinkerImpl::Circular(linker) => linker.write_to_file(),
            LinkerImpl::ReverseUnicast(linker) => linker.write_to_file(),
        }
    }

    fn close_file(self) {
        match self {
            LinkerImpl::Circular(linker) => linker.close(),
            LinkerImpl::ReverseUnicast(linker) => linker.close(),
        }
    }
}

impl LinkerImpl {
    pub(crate) fn new(output_path: &Path, link_settings: &LinkSettings) -> LinkerImpl {
        let output_file = output_path.join(&link_settings.links_file);
        let writer = WriterType::new(&output_file, Self::schema());
        match link_settings.link_model.to_lowercase().as_str() {
            "circular" => LinkerImpl::Circular(CircularLinker::new(writer, link_settings)),
            "reverseunicast" => {
                LinkerImpl::ReverseUnicast(ReverseUnicastLinker::new(writer, link_settings))
            }
            _ => panic!("Invalid linker model"),
        }
    }

    pub(crate) fn calculate_links(
        &mut self,
        source_positions: &AgentIdPos,
        destination_tree: &KdTree<f64, 2>,
        now: TimeMS,
    ) {
        match self {
            LinkerImpl::Circular(circular) => {
                circular.calculate_links(source_positions, destination_tree, now)
            }
            LinkerImpl::ReverseUnicast(unicast) => {
                unicast.calculate_links(source_positions, destination_tree, now)
            }
        }
    }

    pub(crate) fn flush_cache(&mut self) {
        match self {
            LinkerImpl::Circular(circular) => circular.flush(),
            LinkerImpl::ReverseUnicast(unicast) => unicast.flush(),
        }
    }
}

pub(crate) struct CircularLinker {
    writer: WriterType,
    link_settings: LinkSettings,
    writer_cache: WriterCache,
}

impl CircularLinker {
    fn new(writer: WriterType, link_settings: &LinkSettings) -> Self {
        Self {
            writer,
            link_settings: link_settings.clone(),
            writer_cache: WriterCache::new(125000),
        }
    }

    pub(crate) fn calculate_links(
        &mut self,
        source_positions: &AgentIdPos,
        destination_tree: &KdTree<f64, 2>,
        now: TimeMS,
    ) {
        for agent_id_pos in source_positions.iter() {
            if let Some(radius) = self.link_settings.link_radius {
                let neighbours: Vec<NearestNeighbour<f64, u64>> = destination_tree
                    .within::<SquaredEuclidean>(&agent_id_pos.1, radius.as_f64() * radius.as_f64());

                neighbours.into_iter().for_each(|neigh_dist| {
                    if neigh_dist.distance > 0. {
                        self.writer_cache.times.push(now.as_u64());
                        self.writer_cache.sources.push(agent_id_pos.0.as_u64());
                        self.writer_cache.destinations.push(neigh_dist.item);
                        self.writer_cache.distances.push(neigh_dist.distance.sqrt());
                    }
                });
                continue;
            }

            if let Some(count) = self.link_settings.link_count {
                let neighbours = destination_tree
                    .nearest_n::<SquaredEuclidean>(&agent_id_pos.1, count.as_usize());

                neighbours.into_iter().for_each(|neigh_dist| {
                    if neigh_dist.distance > 0. {
                        self.writer_cache.times.push(now.as_u64());
                        self.writer_cache.sources.push(agent_id_pos.0.as_u64());
                        self.writer_cache.destinations.push(neigh_dist.item);
                        self.writer_cache.distances.push(neigh_dist.distance.sqrt());
                    }
                });
            }
        }
    }

    pub(crate) fn write_to_file(&mut self) {
        if self.writer_cache.is_full() {
            debug!("Cache full. Writing to files");
            self.writer
                .record_batch_to_file(&self.writer_cache.as_record_batch());
        }
    }

    pub(crate) fn flush(&mut self) {
        debug!(r"Link calculation done. Flushing the cache to file");
        self.writer
            .record_batch_to_file(&self.writer_cache.as_record_batch());
    }

    pub(crate) fn close(self) {
        self.writer.close();
    }
}

pub struct ReverseUnicastLinker {
    writer: WriterType,
    _link_settings: LinkSettings,
    writer_cache: WriterCache,
}

impl ReverseUnicastLinker {
    fn new(writer: WriterType, link_settings: &LinkSettings) -> Self {
        debug!("Unicast linker selected, link radius and count are not considered");
        Self {
            writer,
            _link_settings: link_settings.to_owned(),
            writer_cache: WriterCache::new(125000),
        }
    }

    fn calculate_links(
        &mut self,
        source_positions: &AgentIdPos,
        destination_tree: &KdTree<f64, 2>,
        now: TimeMS,
    ) {
        for agent_id_pos in source_positions.iter() {
            // Get the nearest neighbor.
            let neighbours: Vec<NearestNeighbour<f64, u64>> =
                destination_tree.nearest_n::<SquaredEuclidean>(&agent_id_pos.1, 1);

            neighbours.into_iter().for_each(|neigh_dist| {
                let mut source = agent_id_pos.0.as_u64();
                let mut destination = neigh_dist.item;
                mem::swap(&mut source, &mut destination);

                if neigh_dist.distance > 0. {
                    self.writer_cache.times.push(now.as_u64());
                    self.writer_cache.sources.push(source);
                    self.writer_cache.destinations.push(destination);
                    self.writer_cache.distances.push(neigh_dist.distance.sqrt());
                }
            });
            continue;
        }
    }

    fn write_to_file(&mut self) {
        if self.writer_cache.is_full() {
            debug!("Cache full. Writing to files");
            self.writer
                .record_batch_to_file(&self.writer_cache.as_record_batch());
        }
    }

    fn flush(&mut self) {
        debug!(r"Link calculation done. Flushing the cache to file");
        self.writer
            .record_batch_to_file(&self.writer_cache.as_record_batch());
    }

    fn close(self) {
        self.writer.close();
    }
}
