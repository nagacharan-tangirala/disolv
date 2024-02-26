pub mod base;
pub mod builder;
pub mod logger;

use advaitars_core::runner::run_simulation;
use clap::Parser;

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
    let mut scheduler: DScheduler = builder.build();
    run_simulation(&mut scheduler);
    let elapsed = start.elapsed();
    println!("Simulation finished in {} ms.", elapsed.as_millis());
}
