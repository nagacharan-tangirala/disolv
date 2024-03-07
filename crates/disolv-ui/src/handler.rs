use crate::content::{LinkContent, SimContent};
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};

/// Terminal events.
#[derive(Clone, Copy, Debug)]
pub enum Message {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    CurrentTime(u64),
    Quit,
}

/// Handles the key events and updates the state of [`SimContent`].
pub fn handle_sim_key_events(key_event: KeyEvent, content: &mut SimContent) {
    match key_event.code {
        // Other handlers you could add here.
        KeyCode::Esc | KeyCode::Char('q') => content.quit(),
        _ => {}
    }
}

/// Handles the key events and updates the state of [`LinkContent`].
pub fn handle_link_key_events(key_event: KeyEvent, content: &mut LinkContent) {
    match key_event.code {
        // Other handlers you could add here.
        KeyCode::Esc | KeyCode::Char('q') => content.quit(),
        _ => {}
    }
}
