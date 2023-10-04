use pavenet_config::config::base::MapState;

pub trait Mapper {
    fn map_state(&self) -> MapState;
    fn set_map_state(&mut self, map_state: MapState);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_map_state() {
        let map_state = MapState::default();
        assert_eq!(map_state.pos.x, 0.0);
        assert_eq!(map_state.pos.y, 0.0);
        assert_eq!(map_state.z, 0.0);
        assert_eq!(map_state.velocity, 0.0);
        assert_eq!(map_state.road_id, 0);
    }

    #[test]
    fn map_state_builder() {
        let map_state = MapState::builder()
            .pos(Point2D::builder().x(1.0).y(2.0).build().unwrap())
            .z(3.0)
            .velocity(4.0)
            .road_id(5)
            .build()
            .unwrap();
        assert_eq!(map_state.pos.x, 1.0);
        assert_eq!(map_state.pos.y, 2.0);
        assert_eq!(map_state.z, 3.0);
        assert_eq!(map_state.velocity, 4.0);
        assert_eq!(map_state.road_id, 5);
    }
}
