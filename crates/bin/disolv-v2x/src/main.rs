use clap::Parser;

use disolv_runner::runner::run_simulation;

use crate::simulation::builder::SimulationBuilder;

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

pub(crate) mod models;
pub(crate) mod simulation;
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
    let scheduler = builder.build();
    run_simulation(scheduler, builder.metadata(), builder.renderer());
    let elapsed = start.elapsed();
    println!("Simulation finished in {} ms.", elapsed.as_millis());
}
