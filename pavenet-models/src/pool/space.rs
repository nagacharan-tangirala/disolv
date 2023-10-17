use crate::model::PoolModel;
use hashbrown::{HashMap, HashSet};
use log::error;
use pavenet_core::enums::MobilityType;
use pavenet_core::node::ids::cell::CellId;
use pavenet_core::structs::{MapState, Point2D};
use pavenet_core::types::{NodeId, TimeStamp};
use pavenet_input::input::mobility::{
    MapFetcher, MapReaderType, MapStateReader, MapStateStreamer, TraceMap,
};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct FieldSettings {
    pub width: f32,
    pub height: f32,
    pub cell_size: f32,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct SpaceSettings {
    pub mobility_type: MobilityType,
    pub is_streaming: bool,
    pub trace_file: String,
}

pub struct Space {
    width: f32,
    height: f32,
    cell_size: f32,
    reader: MapReaderType,
    pub map_states: TraceMap,
    pub map_cache: HashMap<NodeId, MapState>,
    cell2node: HashMap<CellId, HashSet<NodeId>>,
    node2cell: HashMap<NodeId, CellId>,
}

impl PoolModel for Space {
    fn init(&mut self, step: TimeStamp) {
        self.map_states = match self.reader {
            MapReaderType::File(ref mut reader) => {
                reader.fetch_traffic_data(step).unwrap_or_default()
            }
            MapReaderType::Stream(ref mut reader) => {
                reader.fetch_traffic_data(step).unwrap_or_default()
            }
        };
    }

    fn stream_data(&mut self, step: TimeStamp) {
        match self.reader {
            MapReaderType::Stream(ref mut reader) => {
                self.map_states = reader.fetch_traffic_data(step).unwrap_or_default();
            }
            _ => {}
        }
    }

    fn refresh_cache(&mut self, step: TimeStamp) {
        self.map_cache = match self.map_states.remove(&step) {
            Some(traces) => traces,
            None => HashMap::new(),
        };
    }
}

impl Space {
    pub fn builder(config_path: &PathBuf) -> SpaceBuilder {
        SpaceBuilder::new(config_path.clone())
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

#[derive(Default)]
pub struct SpaceBuilder {
    config_path: PathBuf,
    streaming_step: TimeStamp,
    field_settings: FieldSettings,
    space_settings: SpaceSettings,
}

impl SpaceBuilder {
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            ..Default::default()
        }
    }

    pub fn streaming_step(mut self, streaming_step: TimeStamp) -> Self {
        self.streaming_step = streaming_step;
        self
    }

    pub fn field_settings(mut self, field_settings: FieldSettings) -> Self {
        self.field_settings = field_settings;
        self
    }

    pub fn space_settings(mut self, space_settings: SpaceSettings) -> Self {
        self.space_settings = space_settings;
        self
    }

    pub fn build(self) -> Space {
        let file_path = self.config_path.join(&self.space_settings.trace_file);

        let map_state_reader: MapReaderType = match self.space_settings.is_streaming {
            true => MapReaderType::Stream(
                MapStateStreamer::builder()
                    .file_path(PathBuf::from(file_path))
                    .streaming_step(self.streaming_step)
                    .build(),
            ),
            false => MapReaderType::File(
                MapStateReader::builder()
                    .file_path(PathBuf::from(file_path))
                    .build(),
            ),
        };

        Space {
            width: self.field_settings.width,
            height: self.field_settings.height,
            cell_size: self.field_settings.cell_size,
            reader: map_state_reader,
            map_states: HashMap::default(),
            map_cache: HashMap::default(),
            cell2node: HashMap::default(),
            node2cell: HashMap::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pavenet_core::structs::Point2D;
    use pavenet_core::types::{RoadId, Velocity};

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
