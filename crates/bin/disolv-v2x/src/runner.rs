use std::{io, thread};
use std::sync::mpsc;
use std::time::Duration;

use crossterm::event::{self, Event as CrosstermEvent};
use log::info;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::scheduler::Scheduler;
use crate::tui::{handle_sim_key_events, Tui};
use crate::ui::{Message, SimContent, SimUIMetadata};

pub fn run_simulation<S>(mut scheduler: S, metadata: SimUIMetadata)
where
    S: Scheduler,
{
    let (sender_ui, receiver_ui) = mpsc::sync_channel(0);
    let sender = sender_ui.clone();
    let terminal_sender = sender_ui.clone();
    let duration = scheduler.duration().as_u64();

    thread::scope(|s| {
        s.spawn(move || {
            let mut ui_content = SimContent::new(duration, metadata);
            let backend = CrosstermBackend::new(io::stderr());
            let terminal = Terminal::new(backend).expect("failed to create terminal");
            let mut tui = Tui::new(terminal);
            tui.init().expect("failed to initialize the terminal");

            while ui_content.running {
                tui.draw_sim_ui(&mut ui_content).expect("failed to draw");
                match receiver_ui.recv() {
                    Ok(message) => match message {
                        Message::CurrentTime(now) => ui_content.update_now(now),
                        Message::Quit => ui_content.quit(),
                        Message::Key(key_event) => {
                            handle_sim_key_events(key_event, &mut ui_content)
                        }
                        Message::Mouse(_) => {}
                        Message::Resize(_, _) => {}
                    },
                    Err(_) => panic!("Error receiving message"),
                }
            }
            tui.exit().expect("failed to exit");
        });

        let end_time = scheduler.duration().as_u64();
        s.spawn(move || {
            thread::scope(|s2| {
                s2.spawn(move || {
                    let tick_rate = Duration::from_millis(500);
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
                            sender.send(m).expect("failed to send terminal events");
                        }
                    }
                });
            });
            let mut now = 0;
            scheduler.initialize();
            while now < end_time {
                scheduler.activate();
                scheduler.collect_stats();
                now = scheduler.trigger().as_u64();
                match terminal_sender.send(Message::CurrentTime(now)) {
                    Ok(_) => {}
                    Err(_) => {
                        info!("User must have requested to quit, terminating at {}", now);
                        scheduler.terminate();
                        return;
                    }
                };
            }
            scheduler.terminate();
            sender_ui.send(Message::Quit).unwrap();
        });
    });
}
