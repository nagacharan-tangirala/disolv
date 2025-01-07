use disolv_models::device::mobility::road::RoadId;
use disolv_models::device::mobility::velocity::Velocity;
use disolv_models::device::mobility::{MapState, Point2D};

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
