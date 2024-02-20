pub mod base;
pub mod builder;
pub mod logger;
#[cfg(any(feature = "visualization", feature = "visualization_wasm"))]
pub mod vis;

use clap::Parser;

#[cfg(not(any(feature = "visualization", feature = "visualization_wasm")))]
use krabmaga::*;

pub static DISCRETIZATION: f32 = 100.0;
use builder::advaitarsBuilder;

#[derive(Parser, Debug)]
#[command(author, version, long_about = None)]
struct CliArgs {
    #[arg(short = 'c', long, value_name = "CONFIG_FILE")]
    config: String,
}

/*
Main used when only the simulation should run, without any visualization.
*/
#[cfg(not(any(feature = "visualization", feature = "visualization_wasm")))]
fn main() {
    let args = CliArgs::parse();
    let start = std::time::Instant::now();
    let mut builder = advaitarsBuilder::new(&args.config);
    let sim_engine: DEngine = builder.build();
    let duration = builder.duration().as_u64();
    let step_size = builder.step_size().as_f32();
    simulate!(sim_engine, duration, step_size, 1);
    let elapsed = start.elapsed();
    println!(
        "Simulation finished in {}.{:03} seconds.",
        elapsed.as_secs(),
        elapsed.subsec_millis()
    );
}

// Visualization specific imports
use crate::builder::DEngine;
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
