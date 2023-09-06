// Global imports (needed for the simulation to run)
mod data;
mod device;
mod models;
mod sim;
mod utils;
use crate::sim::builder::PavenetBuilder;
use crate::sim::core::Core;
use std::env;

#[cfg(not(any(feature = "visualization", feature = "visualization_wasm")))]
use krabmaga::*;

pub static DISCRETIZATION: f32 = 100.0;

/*
Main used when only the simulation should run, without any visualization.
*/
#[cfg(not(any(feature = "visualization", feature = "visualization_wasm")))]
fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 => {
            let config_file = &args[1];
            let simulation_core: Core = PavenetBuilder::new(config_file).build();
            let duration = simulation_core.get_duration();
            println!("Running the simulation for {} steps", duration);
            simulate!(simulation_core, duration, 1);
        }
        _ => {
            println!("Invalid number of arguments. Usage: pavenet config_file");
            std::process::exit(1);
        }
    }

    if let Some(config_file) = args.get(1) {
    } else {
        println!("Invalid number of arguments. Usage: pavenet config_file");
        std::process::exit(1);
    }
}

#[cfg(any(feature = "visualization", feature = "visualization_wasm"))]
mod visualization;

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
