use clap::Parser;

use crate::simulation::builder::SimulationBuilder;
use crate::simulation::runner::run_simulation;

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

mod simulation;
pub(crate) mod v2x;

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
    let scheduler = builder.build_with_map();
    run_simulation(scheduler, builder.metadata());
    let elapsed = start.elapsed();
    println!("Simulation finished in {} ms.", elapsed.as_millis());
}
