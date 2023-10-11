mod config;
mod scenario;
#[cfg(any(feature = "visualization", feature = "visualization_wasm"))]
pub mod vis;

use clap::Parser;

#[cfg(not(any(feature = "visualization", feature = "visualization_wasm")))]
use krabmaga::*;

pub static DISCRETIZATION: f32 = 100.0;

#[derive(Parser, Debug)]
#[command(author, version, long_about = None)]
struct CliArgs {
    #[arg(short = 'b', long, value_name = "BASE_CONFIG_FILE")]
    base: String,
}

/*
Main used when only the simulation should run, without any visualization.
*/
#[cfg(not(any(feature = "visualization", feature = "visualization_wasm")))]
fn main() {
    let args = CliArgs::parse();
    let simulation_core: Core = PavenetBuilder::new(&args.base, &args.dynamic).build();
    let duration = simulation_core.get_duration();
    println!("Running the simulation for {} steps", duration);
    simulate!(simulation_core, duration, 1);
}

// Visualization specific imports
#[cfg(any(feature = "visualization", feature = "visualization_wasm"))]
use {
    crate::visualization::sea_vis::SeaVis, krabmaga::bevy::prelude::Color,
    krabmaga::visualization::visualization::Visualization,
};

// Main used when a visualization feature is applied.
#[cfg(any(feature = "visualization", feature = "visualization_wasm"))]
fn main() {
    // Initialize the simulation and its visualization here.

    let num_agents = 10;
    let dim: (f32, f32) = (400., 400.);

    let state = Sea::new(dim, num_agents);
    Visualization::default()
        .with_window_dimensions(800., 800.)
        .with_simulation_dimensions(dim.0, dim.1)
        .with_background_color(Color::BLUE)
        .with_name("Template")
        .start::<SeaVis, Sea>(SeaVis, state);
}
