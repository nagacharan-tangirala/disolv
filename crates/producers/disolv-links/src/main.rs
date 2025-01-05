use std::{io, thread};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use clap::Parser;
use crossterm::event::{self, Event as CrosstermEvent};
use log::{debug, info};
use ratatui::{Frame, Terminal};
use ratatui::backend::CrosstermBackend;

use disolv_output::terminal::{handle_sim_key_events, TerminalUI};
use disolv_output::ui::{Message, Renderer, SimContent};
use disolv_runner::runner::{add_event_listener, add_event_poller};

use crate::simulation::config::{Config, read_config};
use crate::simulation::finder::LinkFinder;
use crate::simulation::ui::SimRenderer;

mod links;
mod simulation;

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
    let finder = LinkFinder::new(config, file_path);
    generate_links(finder);
    let elapsed = start.elapsed();
    println!("Link calculation finished in {} ms.", elapsed.as_millis());
}

fn generate_links(mut link_finder: LinkFinder) {
    let (sender_ui, receiver_ui) = mpsc::sync_channel(0);
    let sender = sender_ui.clone();
    let terminal_sender = sender_ui.clone();
    let duration = link_finder.end.as_u64();
    let metadata = link_finder.build_link_metadata();
    let renderer = SimRenderer::new();
    thread::scope(|s| {
        s.spawn(move || {
            add_event_listener(receiver_ui, duration, metadata, renderer);
        });

        s.spawn(move || {
            thread::scope(|s2| {
                s2.spawn(move || add_event_poller(&sender));
            });
            link_finder.initialize();
            debug!("{} {}", link_finder.start, link_finder.end);
            let mut now = link_finder.start;
            while now < link_finder.end {
                link_finder.find_links_at(now);
                match terminal_sender.send(Message::CurrentTime(now.as_u64())) {
                    Ok(_) => {}
                    Err(_) => {
                        info!("User must have requested to quit, terminating at {}", now);
                        link_finder.complete();
                        return;
                    }
                };
                now += link_finder.step_size;
            }
            link_finder.complete();
            sender_ui
                .send(Message::Quit)
                .expect("Failed to send quit message");
        });
    });
}
