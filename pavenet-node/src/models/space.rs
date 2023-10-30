use crate::d_model::BucketModel;
use pavenet_core::bucket::TimeS;
use pavenet_core::entity::id::NodeId;
use pavenet_core::mobility::cell::CellId;
use pavenet_core::mobility::{MapState, MobilityType, Point2D};
use pavenet_engine::hashbrown::{HashMap, HashSet};
use pavenet_input::mobility::data::{
    MapFetcher, MapReader, MapStateReader, MapStateStreamer, TraceMap,
};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Space {
    width: f32,
    height: f32,
    cell_size: f32,
    cell2node: HashMap<CellId, HashSet<NodeId>>,
    node2cell: HashMap<NodeId, CellId>,
}

impl Space {
    pub fn new(width: f32, height: f32, cell_size: f32) -> Self {
        Self {
            width,
            height,
            cell_size,
            cell2node: HashMap::new(),
            node2cell: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node_id: NodeId, location: &Point2D) {
        let cell_id = self.get_cell_id(location);
        let old_cell_id = self.node2cell.entry(node_id).or_default();
        if *old_cell_id != cell_id {
            if let Some(nodes) = self.cell2node.get_mut(&cell_id) {
                nodes.remove(&node_id);
            }
            self.add_node_to_cell(node_id, cell_id);
            self.node2cell.entry(node_id).and_modify(|e| *e = cell_id);
        }
    }

    pub fn nodes(&self, cell_id: CellId) -> Option<&HashSet<NodeId>> {
        self.cell2node.get(&cell_id)
    }

    pub fn cell_id(&self, node_id: NodeId) -> Option<&CellId> {
        self.node2cell.get(&node_id)
    }

    #[inline]
    fn add_node_to_cell(&mut self, node_id: NodeId, cell_id: CellId) {
        self.cell2node
            .entry(cell_id)
            .or_insert_with(HashSet::new)
            .insert(node_id);
    }

    #[inline]
    fn get_cell_id(&self, location: &Point2D) -> CellId {
        let cell_x = (location.x / self.cell_size).round();
        let cell_y = (location.y / self.cell_size).round();
        CellId::from(cell_x + (cell_y * self.width))
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct FieldSettings {
    pub width: f32,
    pub height: f32,
    pub cell_size: f32,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct MapperSettings {
    pub mobility_type: MobilityType,
    pub is_streaming: bool,
    pub trace_file: String,
}

#[derive(Clone)]
pub(crate) struct Mapper {
    reader: MapReader,
    map_states: TraceMap,
    map_cache: HashMap<NodeId, MapState>,
}

impl BucketModel for Mapper {
    fn init(&mut self, step: TimeS) {
        self.map_states = match self.reader {
            MapReader::File(ref mut reader) => reader.fetch_traffic_data(step).unwrap_or_default(),
            MapReader::Stream(ref mut reader) => {
                reader.fetch_traffic_data(step).unwrap_or_default()
            }
        };
    }

    fn stream_data(&mut self, step: TimeS) {
        match self.reader {
            MapReader::Stream(ref mut reader) => {
                self.map_states = reader.fetch_traffic_data(step).unwrap_or_default();
            }
            _ => {}
        }
    }

    fn refresh_cache(&mut self, step: TimeS) {
        self.map_cache = match self.map_states.remove(&step) {
            Some(traces) => traces,
            None => HashMap::new(),
        };
    }
}

impl Mapper {
    pub fn builder(config_path: &PathBuf) -> MapperBuilder {
        MapperBuilder::new(config_path.clone())
    }

    pub fn map_state_of(&mut self, node_id: NodeId) -> Option<MapState> {
        self.map_cache.remove(&node_id)
    }
}

#[derive(Default)]
pub struct MapperBuilder {
    config_path: PathBuf,
    streaming_step: TimeS,
    field_settings: FieldSettings,
    space_settings: MapperSettings,
}

impl MapperBuilder {
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            ..Default::default()
        }
    }

    pub fn streaming_step(mut self, streaming_step: TimeS) -> Self {
        self.streaming_step = streaming_step;
        self
    }

    pub fn field_settings(mut self, field_settings: FieldSettings) -> Self {
        self.field_settings = field_settings;
        self
    }

    pub fn space_settings(mut self, space_settings: MapperSettings) -> Self {
        self.space_settings = space_settings;
        self
    }

    pub fn build(self) -> Mapper {
        let file_path = self.config_path.join(&self.space_settings.trace_file);

        let map_state_reader: MapReader = match self.space_settings.is_streaming {
            true => MapReader::Stream(
                MapStateStreamer::builder()
                    .file_path(PathBuf::from(file_path))
                    .streaming_step(self.streaming_step)
                    .build(),
            ),
            false => MapReader::File(
                MapStateReader::builder()
                    .file_path(PathBuf::from(file_path))
                    .build(),
            ),
        };

        Mapper {
            reader: map_state_reader,
            map_states: HashMap::default(),
            map_cache: HashMap::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pavenet_core::mobility::road::RoadId;
    use pavenet_core::mobility::velocity::Velocity;

    #[test]
    fn default_map_state() {
        let map_state = MapState::default();
        assert_eq!(map_state.pos.x, 0.0);
        assert_eq!(map_state.pos.y, 0.0);
        assert_eq!(map_state.z, Some(0.0));
        assert_eq!(map_state.velocity, Some(Velocity::from(0.0)));
        assert_eq!(map_state.road_id, Some(RoadId::from(0)));
    }

    #[test]
    fn map_state_builder() {
        let map_state = MapState::builder()
            .pos(Point2D::builder().x(1.0).y(2.0).build())
            .z(Some(3.0f32))
            .velocity(Some(Velocity::from(4.0)))
            .road_id(Some(RoadId::from(5)))
            .build();
        assert_eq!(map_state.pos.x, 1.0);
        assert_eq!(map_state.pos.y, 2.0);
        assert_eq!(map_state.z, Some(3.0));
        assert_eq!(map_state.velocity, Some(Velocity::from(4.0)));
        assert_eq!(map_state.road_id, Some(RoadId::from(5)));
    }
}
