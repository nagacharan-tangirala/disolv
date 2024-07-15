#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

mod base;
mod builder;
mod logger;
mod bucket;
mod device;
mod linker;
mod space;

use clap::Parser;
use disolv_core::runner::run_simulation;

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
    let scheduler = builder.build_with_map();
    run_simulation(scheduler, builder.metadata());
    let elapsed = start.elapsed();
    println!("Simulation finished in {} ms.", elapsed.as_millis());
}
