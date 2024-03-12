#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

pub mod base;
pub mod builder;
pub mod logger;

use clap::Parser;
use disolv_core::runner::run_simulation;

use crate::builder::DScheduler;
use builder::SimulationBuilder;

#[derive(Parser, Debug)]
#[command(author, version, long_about = None)]
struct CliArgs {
    #[arg(short = 'c', long, value_name = "CONFIG_FILE")]
    config: String,
}

fn main() {
    let args = CliArgs::parse();
    let start = std::time::Instant::now();
    let mut builder = SimulationBuilder::new(&args.config);
    let scheduler: DScheduler = builder.build();
    run_simulation(scheduler, builder.metadata());
    let elapsed = start.elapsed();
    println!("Simulation finished in {} ms.", elapsed.as_millis());
}
