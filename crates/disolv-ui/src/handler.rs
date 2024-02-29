use crate::content::Content;
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

/// Handles the key events and updates the state of [`Content`].
pub fn handle_key_events(key_event: KeyEvent, content: &mut Content) {
    match key_event.code {
        // Other handlers you could add here.
        KeyCode::Esc | KeyCode::Char('q') => content.quit(),
        _ => {}
    }
}
