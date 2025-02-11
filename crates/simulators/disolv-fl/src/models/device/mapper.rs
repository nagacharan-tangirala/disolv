use std::path::{Path, PathBuf};

use hashbrown::{HashMap, HashSet};
use serde::Deserialize;
use typed_builder::TypedBuilder;

use disolv_core::agent::AgentId;
use disolv_core::bucket::TimeMS;
use disolv_core::model::BucketModel;
use disolv_input::mobility::{MapReader, TraceMap};
use disolv_models::device::mobility::cell::CellId;
use disolv_models::device::mobility::{MapState, MobilityType, Point2D};

#[derive(Clone, Debug, TypedBuilder)]
pub struct GeoMap {
    width: f64,
    height: f64,
    cell_size: f64,
    #[builder(default)]
    cell2agent: HashMap<CellId, HashSet<AgentId>>,
    #[builder(default)]
    agent2cell: HashMap<AgentId, CellId>,
}

impl GeoMap {
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
pub(crate) struct MobilitySettings {
    pub(crate) mobility_type: MobilityType,
    pub(crate) is_streaming: bool,
    pub mobility_step: Option<TimeMS>,
    pub trace_file: String,
}

#[derive(Clone)]
pub(crate) struct GeoMapper {
    reader: MapReader,
    map_states: TraceMap,
    map_cache: HashMap<AgentId, MapState>,
}

impl BucketModel for GeoMapper {
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

impl GeoMapper {
    pub(crate) fn builder(config_path: &Path) -> MapperBuilder {
        MapperBuilder::new(config_path)
    }

    pub(crate) fn map_state_of(&mut self, agent_id: AgentId) -> Option<MapState> {
        self.map_cache.remove(&agent_id)
    }
}

#[derive(Default)]
pub(crate) struct MapperBuilder {
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

    pub fn build(self) -> GeoMapper {
        let file_path = self.config_path.join(&self.space_settings.trace_file);
        let map_reader = MapReader::builder()
            .file_path(file_path)
            .streaming_step(self.streaming_step)
            .is_streaming(self.space_settings.is_streaming)
            .build();

        GeoMapper {
            reader: map_reader,
            map_states: HashMap::default(),
            map_cache: HashMap::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use disolv_models::device::mobility::road::RoadId;
    use disolv_models::device::mobility::velocity::Velocity;

    use super::*;

    #[test]
    fn map_state_builder() {
        let map_state = MapState::builder()
            .pos(Point2D::builder().x(1.0).y(2.0).build())
            .z(Some(3.0f64))
            .velocity(Some(Velocity::from(4.0)))
            .road_id(Some(RoadId::from(5u32)))
            .build();
        assert_eq!(map_state.pos.x, 1.0);
        assert_eq!(map_state.pos.y, 2.0);
        assert_eq!(map_state.z, Some(3.0));
        assert_eq!(map_state.velocity, Some(Velocity::from(4.0)));
        assert_eq!(map_state.road_id, Some(RoadId::from(5u32)));
    }
}
