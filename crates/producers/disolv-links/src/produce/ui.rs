use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;

use disolv_output::ui::{Renderer, SimContent};

pub struct SimRenderer {}

impl SimRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Renderer for SimRenderer {
    fn render_sim_ui(&self, content: &mut SimContent, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(frame.area());

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
}
