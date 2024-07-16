use std::error;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};

pub type ContentResult<T> = Result<T, Box<dyn error::Error>>;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Message {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    CurrentTime(u64),
    Quit,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct UIMetadata {
    pub input_file: String,
    pub output_path: String,
}

#[derive(Debug, Default)]
pub(crate) struct Content {
    pub running: bool,
    pub total_steps: u64,
    pub now: u64,
    pub metadata: UIMetadata,
}

impl Content {
    pub fn new(total_steps: u64, metadata: UIMetadata) -> Self {
        Self {
            total_steps,
            running: true,
            metadata,
            ..Self::default()
        }
    }
    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn update_now(&mut self, now: u64) {
        self.now = now;
    }

    pub fn completion(&self) -> f64 {
        self.now as f64 / self.total_steps as f64
    }
}

pub fn render_ui(content: &mut Content, frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(frame.size());

    let completion = content.completion();
    let progress_text = format!(
        "Time Step: {} / {} steps. {:.2}%. ",
        content.now,
        content.total_steps,
        content.completion() * 100.0
    );
    frame.render_widget(
        Gauge::default()
            .gauge_style(
                Style::default()
                    .fg(Color::Magenta)
                    .bg(Color::Black)
                    .add_modifier(Modifier::ITALIC),
            )
            .label(progress_text)
            .ratio(completion)
            .use_unicode(true)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Link Calculation Progress")
                    .title_alignment(Alignment::Center),
            ),
        layout[0],
    );

    let simulation_details = format!(
        "Input File: {}\n\
        Output Path: {}\n\
        ",
        content.metadata.input_file, content.metadata.output_path,
    );
    frame.render_widget(
        Paragraph::new(simulation_details)
            .block(Block::default().borders(Borders::ALL).title("More details"))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Left),
        layout[1],
    );
}
