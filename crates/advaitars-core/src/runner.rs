use crate::agent::Agent;
use crate::bucket::Bucket;
use crate::scheduler::{DefaultScheduler, Scheduler};
use advaitars_ui::content::Content;
use advaitars_ui::handler::{handle_key_events, Message};
use advaitars_ui::tui::Tui;
use crossterm::event::{self, Event as CrosstermEvent, Event, KeyCode, KeyEvent, MouseEvent};
use log::{debug, info};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{io, thread};

pub fn run_simulation<A, B>(scheduler: &mut DefaultScheduler<A, B>)
where
    A: Agent<B>,
    B: Bucket,
{
    let (sender_ui, receiver_ui) = mpsc::sync_channel(1);
    let sender = sender_ui.clone();
    let terminal_event_sender = sender_ui.clone();
    let duration = scheduler.duration.as_u64();
    thread::scope(|s| {
        s.spawn(move || {
            let mut ui_content = Content::new(duration);

            // Initialize the terminal user interface.
            let backend = CrosstermBackend::new(io::stderr());
            let terminal = Terminal::new(backend).expect("failed to create terminal");
            let mut tui = Tui::new(terminal);
            tui.init().expect("failed to initialize the terminal");

            // Start the main loop.
            while ui_content.running {
                // Render the user interface.
                tui.draw(&mut ui_content).expect("failed to draw");

                // Handle the messages.
                match receiver_ui.recv() {
                    Ok(message) => match message {
                        Message::CurrentTime(now) => ui_content.update_now(now),
                        Message::Quit => ui_content.quit(),
                        Message::Key(key_event) => handle_key_events(key_event, &mut ui_content),
                        Message::Mouse(_) => {}
                        Message::Resize(_, _) => {}
                    },
                    Err(_) => panic!("Error receiving message"),
                }
            }
            tui.exit().expect("failed to exit");
        });

        let end_time = scheduler.duration.as_u64();
        let step_size = scheduler.step_size.as_u64();
        s.spawn(move || {
            if step_size == 0 {
                panic!("Step size cannot be zero");
            }
            let mut now = 0;
            scheduler.initialize();
            while now < end_time {
                scheduler.activate();
                scheduler.collect_stats();
                now = scheduler.trigger().as_u64();

                match terminal_event_sender.send(Message::CurrentTime(now)) {
                    Ok(_) => {}
                    Err(err) => {
                        info!("User must have requested to quit, terminating at {}", now);
                        scheduler.terminate();
                        return;
                    }
                };
            }
            scheduler.terminate();
            sender_ui.send(Message::Quit).unwrap();
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
                        Err(err) => return,
                    }
                }
            }
        });
    });
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::agent::tests::TDevice;
    use crate::bucket::tests::MyBucket;
    use crate::bucket::TimeMS;
    use crate::core::tests::create_core;
    use crate::scheduler::tests::create_scheduler;

    #[test]
    fn test_run_simulation() {
        let mut scheduler = create_scheduler();
        assert_eq!(scheduler.now, TimeMS::from(0));
        run_simulation(&mut scheduler);
        assert_eq!(scheduler.now, scheduler.duration);
    }
}
