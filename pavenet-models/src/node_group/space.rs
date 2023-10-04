use hashbrown::{HashMap, HashSet};
use log::error;
use pavenet_config::config::base::{FieldSettings, MapState, MapStateSettings, Point2D};
use pavenet_config::types::ids::cell::CellId;
use pavenet_config::types::ids::node::NodeId;
use pavenet_config::types::ts::TimeStamp;
use pavenet_io::input::maps::{
    MapFetcher, MapReaderType, MapStateReader, MapStateStreamer, TraceMap,
};
use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct Space {
    width: f32,
    height: f32,
    cell_size: f32,
    map_state_reader: MapReaderType,
    pub map_states: TraceMap,
    pub map_cache: HashMap<NodeId, MapState>,
    cell2node: HashMap<CellId, HashSet<NodeId>>,
    node2cell: HashMap<NodeId, CellId>,
}

impl Space {
    pub fn builder(config_path: &PathBuf) -> SpaceBuilder {
        SpaceBuilder::new(config_path.clone())
    }

    pub fn init(&mut self, step: TimeStamp) {
        self.read_map_state(&mut self.map_state_reader, step);
    }

    pub fn add_node(&mut self, node_id: NodeId, location: &Point2D) {
        let cell_id = self.get_cell_id(location);
        let old_cell_id = self.node2cell.entry(node_id).or_default();
        if *old_cell_id != cell_id {
            self.remove_node_from_cell(node_id, *old_cell_id);
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

    pub fn stream_map_states(&mut self, step: TimeStamp) {
        match self.map_state_reader {
            MapReaderType::Stream(ref mut reader) => {
                self.read_map_state(reader, step);
            }
            _ => {}
        }
    }

    pub fn update_map_cache(&mut self, step: TimeStamp) {
        self.map_cache = match self.map_states.remove(&step) {
            Some(traces) => traces,
            None => HashMap::new(),
        };
    }

    fn read_map_state(&mut self, reader: &mut dyn MapFetcher, step: TimeStamp) {
        self.map_states = match reader.fetch_traffic_data(step) {
            Ok(map) => map,
            Err(e) => {
                error!("Error reading map state: {}", e);
                HashMap::new()
            }
        };
    }

    #[inline]
    fn remove_node_from_cell(&mut self, node_id: NodeId, cell_id: CellId) {
        if let Some(nodes) = self.cell2node.get_mut(&cell_id) {
            nodes.remove(&node_id);
        }
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

struct SpaceBuilder {
    config_path: PathBuf,
    streaming_interval: TimeStamp,
    field_settings: FieldSettings,
    map_state_reader: MapStateReader,
    map_state_settings: MapStateSettings,
}

impl SpaceBuilder {
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            ..Default::default()
        }
    }

    pub fn streaming_interval(mut self, streaming_interval: TimeStamp) -> Self {
        self.streaming_interval = streaming_interval;
        self
    }

    pub fn field_settings(mut self, field_settings: FieldSettings) -> Self {
        self.field_settings = field_settings;
        self
    }

    pub fn map_settings(mut self, map_state_settings: MapStateSettings) -> Self {
        self.map_state_settings = map_state_settings;
        self
    }

    pub fn build(self) -> Space {
        let file_path = self
            .config_path
            .join(&self.map_state_settings.geo_data_file);

        let map_state_reader: MapReaderType = match self.map_state_settings.is_streaming {
            true => MapReaderType::Stream(MapStateStreamer::new(
                file_path.clone(),
                self.streaming_interval,
            )),
            false => MapReaderType::File(MapStateReader::new(file_path.clone())),
        };

        let mut space = Space {
            width: self.field_settings.width,
            height: self.field_settings.height,
            cell_size: self.field_settings.cell_size,
            map_state_reader,
            ..Default::default()
        };
        space
    }
}
