mod builder;
mod config;
mod linker;
mod logger;
mod reader;

use crate::builder::LinkBuilder;
use crate::config::{read_config, Config};
use clap::Parser;
use crossterm::event::{self, Event as CrosstermEvent};
use disolv_core::tui::{handle_link_key_events, Tui};
use disolv_core::ui::{LinkContent, Message};
use log::info;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use std::{io, thread};

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
    let builder = LinkBuilder::new(config, file_path);
    generate_links(builder);
    let elapsed = start.elapsed();
    println!("Link calculation finished in {} ms.", elapsed.as_millis());
}

fn generate_links(mut builder: LinkBuilder) {
    let (sender_ui, receiver_ui) = mpsc::sync_channel(0);
    let sender = sender_ui.clone();
    let terminal_event_sender = sender_ui.clone();
    let duration = builder.end.as_u64();
    let ui_metadata = builder.build_link_metadata();
    thread::scope(|s| {
        s.spawn(move || {
            let mut ui_content = LinkContent::new(duration, ui_metadata);

            // Initialize the terminal user interface.
            let backend = CrosstermBackend::new(io::stderr());
            let terminal = Terminal::new(backend).expect("failed to create terminal");
            let mut tui = Tui::new(terminal);
            tui.init().expect("failed to initialize the terminal");

            // Start the main loop.
            while ui_content.running {
                // Render the user interface.
                tui.draw_link_ui(&mut ui_content).expect("failed to draw");

                // Handle the messages.
                match receiver_ui.recv() {
                    Ok(message) => match message {
                        Message::CurrentTime(now) => ui_content.update_now(now),
                        Message::Quit => ui_content.quit(),
                        Message::Key(key_event) => {
                            handle_link_key_events(key_event, &mut ui_content)
                        }
                        Message::Mouse(_) => {}
                        Message::Resize(_, _) => {}
                    },
                    Err(_) => panic!("Error receiving message"),
                }
            }
            tui.exit().expect("failed to exit");
        });

        s.spawn(move || {
            if builder.step_size.as_u64() == 0 {
                panic!("Step size cannot be zero");
            }
            builder.initialize();
            let mut now = builder.start;
            while now < builder.end {
                builder.build_links_at(now);
                match terminal_event_sender.send(Message::CurrentTime(now.as_u64())) {
                    Ok(_) => {}
                    Err(_) => {
                        info!("User must have requested to quit, terminating at {}", now);
                        builder.complete();
                        return;
                    }
                };
                now += builder.step_size;
            }
            builder.complete();
            sender_ui
                .send(Message::Quit)
                .expect("Failed to send quit message");
        });

        s.spawn(move || loop {
            let tick_rate = Duration::from_millis(1000);
            let mut message = None;
            if event::poll(tick_rate).expect("failed to poll new events") {
                match event::read().expect("unable to read event") {
                    CrosstermEvent::Key(e) => message = Some(Message::Key(e)),
                    CrosstermEvent::Mouse(e) => message = Some(Message::Mouse(e)),
                    CrosstermEvent::Resize(w, h) => message = Some(Message::Resize(w, h)),
                    CrosstermEvent::FocusGained => {}
                    CrosstermEvent::FocusLost => {}
                    CrosstermEvent::Paste(_) => {}
                };

                if let Some(m) = message {
                    match sender.send(m) {
                        Ok(_) => {}
                        Err(_) => return,
                    }
                }
            }
        });
    });
}
