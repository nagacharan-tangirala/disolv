use disolv_v2x::simulation::runner::run_simulation;
use disolv_v2x::simulation::ui::SimUIMetadata;

use crate::scheduler::{create_map_scheduler, create_scheduler};

#[test]
fn test_run_simulation() {
    let scheduler = create_scheduler();
    run_simulation(scheduler, SimUIMetadata::default());
}

#[test]
fn test_run_simulation_with_map() {
    let scheduler = create_map_scheduler();
    run_simulation(scheduler, SimUIMetadata::default());
}
