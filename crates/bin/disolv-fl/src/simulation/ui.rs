use std::error;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Paragraph},
};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Borders, Gauge};

pub type ContentResult<T> = Result<T, Box<dyn error::Error>>;

#[derive(Clone, Copy, Debug)]
pub enum Message {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    CurrentTime(u64),
    Quit,
}

#[derive(Debug, Clone, Default)]
pub struct SimUIMetadata {
    pub scenario: String,
    pub input_file: String,
    pub output_path: String,
    pub log_path: String,
}

/// Application.
#[derive(Debug, Default)]
pub struct SimContent {
    pub running: bool,
    pub total_steps: u64,
    pub now: u64,
    pub metadata: SimUIMetadata,
    pub total_agents: usize,
    pub active_agents: usize,
}

impl SimContent {
    pub fn new(total_steps: u64, metadata: SimUIMetadata) -> Self {
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

/// Renders the user interface widgets.
pub fn render_sim_ui(content: &mut SimContent, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(50),
        ])
        .split(frame.area());

    frame.render_widget(
        Paragraph::new(format!(
            " ===================== d i s o l v - FL =====================\n\
              \n\
              Scenario: {}\n\
              \n\
              ===============================================================
            ",
            content.metadata.scenario
        ))
        .block(
            Block::bordered()
                .title("disolv")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Red).bg(Color::Black))
        .centered(),
        layout[0],
    );

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
                    .fg(Color::LightBlue)
                    .bg(Color::Black)
                    .add_modifier(Modifier::ITALIC),
            )
            .label(progress_text)
            .ratio(completion)
            .use_unicode(true)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Simulation Progress")
                    .title_alignment(Alignment::Center),
            ),
        layout[1],
    );

    let simulation_details = format!(
        "Input File: {}\n\
        Output Path: {}\n\
        Log Path: {}\n\
        Active Agents: {}\n\
        Total Agents: {}\n\
        ",
        content.metadata.input_file,
        content.metadata.output_path,
        content.metadata.log_path,
        content.active_agents,
        content.total_agents
    );
    frame.render_widget(
        Paragraph::new(simulation_details)
            .block(Block::default().borders(Borders::ALL).title("More details"))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Left),
        layout[2],
    );
}
