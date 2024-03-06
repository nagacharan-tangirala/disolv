mod builder;
mod config;
mod finder;
mod linker;
mod logger;
mod reader;

use crate::builder::LinkBuilder;
use crate::config::{read_config, Config};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, long_about = None)]
struct CliArgs {
    #[arg(short = 'c', long, value_name = "Link Configuration File")]
    config: String,
}

fn main() {
    let config_file: String = CliArgs::parse().config;
    let start = std::time::Instant::now();
    let file_path = PathBuf::from(config_file);
    let config: Config = read_config(&file_path);
    let mut builder = LinkBuilder::new(config, file_path);
    builder.initiate();
    builder.build_links();
    let elapsed = start.elapsed();
    println!("Link calculation finished in {} ms.", elapsed.as_millis());
}
