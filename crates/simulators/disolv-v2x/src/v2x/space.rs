use std::path::{Path, PathBuf};

use hashbrown::{HashMap, HashSet};
use serde::Deserialize;
use typed_builder::TypedBuilder;

use disolv_core::agent::AgentId;
use disolv_core::bucket::TimeMS;
use disolv_core::model::BucketModel;
use disolv_input::mobility::{MapReader, TraceMap};
use disolv_models::device::mobility::{MapState, MobilityType, Point2D};
use disolv_models::device::mobility::cell::CellId;

#[derive(Clone, Debug, TypedBuilder)]
pub struct Space {
    width: f64,
    height: f64,
    cell_size: f64,
    #[builder(default)]
    cell2agent: HashMap<CellId, HashSet<AgentId>>,
    #[builder(default)]
    agent2cell: HashMap<AgentId, CellId>,
}

impl Space {
    pub fn add_agent(&mut self, agent_id: AgentId, location: &Point2D) {
        let cell_id = self.get_cell_id(location);
        let old_cell_id = self.agent2cell.entry(agent_id).or_default();
        if *old_cell_id != cell_id {
            if let Some(agents) = self.cell2agent.get_mut(&cell_id) {
                agents.remove(&agent_id);
            }
            self.add_agent_to_cell(agent_id, cell_id);
            self.agent2cell.entry(agent_id).and_modify(|e| *e = cell_id);
        }
    }

    pub fn agents(&self, cell_id: CellId) -> Option<&HashSet<AgentId>> {
        self.cell2agent.get(&cell_id)
    }

    pub fn cell_id(&self, agent_id: AgentId) -> Option<&CellId> {
        self.agent2cell.get(&agent_id)
    }

    #[inline]
    fn add_agent_to_cell(&mut self, agent_id: AgentId, cell_id: CellId) {
        self.cell2agent.entry(cell_id).or_default().insert(agent_id);
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
    pub width: f64,
    pub height: f64,
    pub cell_size: f64,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct MobilitySettings {
    pub mobility_type: MobilityType,
    pub is_streaming: bool,
    pub mobility_step: Option<TimeMS>,
    pub trace_file: String,
}

#[derive(Clone)]
pub struct Mapper {
    reader: MapReader,
    map_states: TraceMap,
    map_cache: HashMap<AgentId, MapState>,
}

impl BucketModel for Mapper {
    fn init(&mut self, step: TimeMS) {
        self.map_states = self.reader.fetch_traffic_data(step);
    }

    fn stream_data(&mut self, step: TimeMS) {
        if self.reader.is_streaming {
            self.map_states = self.reader.fetch_traffic_data(step);
        }
    }

    fn before_agent_step(&mut self, step: TimeMS) {
        self.map_cache = self.map_states.remove(&step).unwrap_or_default()
    }
}

impl Mapper {
    pub fn builder(config_path: &Path) -> MapperBuilder {
        MapperBuilder::new(config_path)
    }

    pub fn map_state_of(&mut self, agent_id: AgentId) -> Option<MapState> {
        self.map_cache.remove(&agent_id)
    }
}

#[derive(Default)]
pub struct MapperBuilder {
    config_path: PathBuf,
    streaming_step: TimeMS,
    field_settings: FieldSettings,
    space_settings: MobilitySettings,
}

impl MapperBuilder {
    pub fn new(config_path: &Path) -> Self {
        Self {
            config_path: PathBuf::from(config_path),
            ..Default::default()
        }
    }

    pub fn streaming_step(mut self, streaming_step: TimeMS) -> Self {
        self.streaming_step = streaming_step;
        self
    }

    pub fn field_settings(mut self, field_settings: FieldSettings) -> Self {
        self.field_settings = field_settings;
        self
    }

    pub fn space_settings(mut self, space_settings: MobilitySettings) -> Self {
        self.space_settings = space_settings;
        self
    }

    pub fn build(self) -> Mapper {
        let file_path = self.config_path.join(&self.space_settings.trace_file);
        let map_reader = MapReader::builder()
            .file_path(file_path)
            .streaming_step(self.streaming_step)
            .is_streaming(self.space_settings.is_streaming)
            .build();

        Mapper {
            reader: map_reader,
            map_states: HashMap::default(),
            map_cache: HashMap::default(),
        }
    }
}

#[cfg(test)]
mod tests {}
