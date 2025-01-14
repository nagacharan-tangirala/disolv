use std::{error, io};
use std::panic;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::Backend;
use ratatui::Terminal;

use crate::ui::{Renderer, SimContent};

pub type ContentResult<T> = Result<T, Box<dyn error::Error>>;

/// Representation of a terminal user interface.
///
/// It is responsible for setting up the terminal,
/// initializing the interface and handling the draw events.
#[derive(Debug)]
pub struct TerminalUI<B: Backend, R: Renderer> {
    /// Interface to the Terminal.
    terminal: Terminal<B>,
    renderer: R,
}

impl<B: Backend, R: Renderer> TerminalUI<B, R> {
    /// Constructs a new instance of [`TerminalUI`].
    pub fn new(terminal: Terminal<B>, renderer: R) -> Self {
        Self { terminal, renderer }
    }

    /// Initializes the terminal interface.
    ///
    /// It enables the raw mode and sets terminal properties.
    pub fn init(&mut self) -> ContentResult<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;

        // Define a custom panic hook to reset the terminal properties.
        // This way, you won't have your terminal messed up if an unexpected error happens.
        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("failed to reset the terminal");
            panic_hook(panic);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    ///
    /// [`Draw`]: ratatui::Terminal::draw
    /// [`rendering`]: crate::ui:render
    pub fn draw_ui(&mut self, app: &mut SimContent) -> ContentResult<()> {
        self.terminal
            .draw(|frame| self.renderer.render_sim_ui(app, frame))?;
        Ok(())
    }

    /// Resets the terminal interface.
    ///
    /// This function is also used for the panic hook to revert
    /// the terminal properties if unexpected errors occur.
    fn reset() -> ContentResult<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    /// Exits the terminal interface.
    ///
    /// It disables the raw mode and reverts back the terminal properties.
    pub fn exit(&mut self) -> ContentResult<()> {
        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

/// Handles the key events and updates the state of [`Content`].
pub fn handle_sim_key_events(key_event: KeyEvent, content: &mut SimContent) {
    match key_event.code {
        // Other handlers you could add here.
        KeyCode::Esc | KeyCode::Char('q') => content.quit(),
        _ => {}
    }
}
