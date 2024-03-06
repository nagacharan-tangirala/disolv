use crate::config::LinkSettings;
use crate::linker::LinkModel;
use crate::reader::AgentIdPos;
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow_array::{ArrayRef, Float64Array, RecordBatch, UInt64Array};
use disolv_core::bucket::TimeMS;
use disolv_input::columns::{AGENT_ID, DISTANCE, TARGET_ID, TIME_STEP};
use kiddo::{KdTree, NearestNeighbour, SquaredEuclidean};
use log::debug;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;
use std::fs::File;
use std::sync::Arc;

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

pub(crate) struct LinkFinder {
    writer: ArrowWriter<File>,
    link_model: LinkModel,
    linker_settings: LinkSettings,
    writer_cache: WriterCache,
}

impl LinkFinder {
    pub(crate) fn new(output_path: &str, link_settings: &LinkSettings) -> Self {
        let output_file = output_path.to_owned() + link_settings.links_file.as_str() + ".parquet";
        let link_model = match link_settings.link_model.to_lowercase().as_str() {
            "circular" => LinkModel::Circular,
            _ => panic!("Invalid linker model"),
        };
        Self {
            link_model,
            writer: Self::create_writer(output_file.as_str()),
            linker_settings: link_settings.clone(),
            writer_cache: WriterCache::new(125000),
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
        let writer = match ArrowWriter::try_new(output_file, SchemaRef::from(schema), Some(props)) {
            Ok(writer) => writer,
            Err(_) => panic!("Failed to create links file writer"),
        };
        writer
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
                        self.writer_cache.distances.push(neigh_dist.distance);
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
                        self.writer_cache.distances.push(neigh_dist.distance);
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
        let _ = self.writer.close().expect("Failed to close the link file");
    }
}
