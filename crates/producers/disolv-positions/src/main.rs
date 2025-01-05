use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use clap::Parser;
use log::info;

use disolv_core::bucket::TimeMS;
use disolv_output::ui::Message;
use disolv_runner::runner::{add_event_listener, add_event_poller};

use crate::produce::config::{Config, read_config};
use crate::produce::parser::TraceParser;
use crate::produce::ui::SimRenderer;

mod activation;
mod produce;
mod rsu;
mod trace;

#[derive(Parser, Debug)]
#[command(author, version, long_about = None)]
struct CliArgs {
    #[arg(short = 'c', long, value_name = "Position Configuration File")]
    config: String,
}

fn main() {
    let config_file: String = CliArgs::parse().config;
    let start = std::time::Instant::now();
    let file_path = PathBuf::from(config_file);
    let config: Config = read_config(&file_path);
    let builder = TraceParser::new(config, file_path);
    generate_positions(builder);
    let elapsed = start.elapsed();
    info!("Trace parsing finished in {} ms.", elapsed.as_millis());
}

fn generate_positions(mut trace_parser: TraceParser) {
    let (sender_ui, receiver_ui) = mpsc::sync_channel(0);
    let sender = sender_ui.clone();
    let terminal_sender = sender_ui.clone();
    let duration = trace_parser.duration.as_u64();
    let metadata = trace_parser.build_trace_metadata();
    let renderer = SimRenderer::new();
    thread::scope(|s| {
        s.spawn(move || {
            add_event_listener(receiver_ui, duration, metadata, renderer);
        });

        s.spawn(move || {
            add_event_poller(&sender);
            trace_parser.initialize();
            let mut now: TimeMS = TimeMS::from(0);
            while now < trace_parser.duration {
                info!("Parsing traces for time {}", now);
                trace_parser.parse_positions_at(now);
                match terminal_sender.send(Message::CurrentTime(now.as_u64())) {
                    Ok(_) => {}
                    Err(_) => {
                        info!("User must have requested to quit, terminating at {}", now);
                        trace_parser.complete();
                        return;
                    }
                };
                now += trace_parser.step_size;
            }
            trace_parser.complete();
            sender_ui
                .send(Message::Quit)
                .expect("Failed to send quit message");
        });
    });
}
