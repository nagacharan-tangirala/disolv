use std::error;

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Borders, Gauge};
use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Paragraph},
    Frame,
};

use disolv_output::ui::{Renderer, SimContent};

pub type ContentResult<T> = Result<T, Box<dyn error::Error>>;

pub struct SimRenderer {}

impl SimRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Renderer for SimRenderer {
    /// Application.
    /// Renders the user interface widgets.
    fn render_sim_ui(&self, content: &mut SimContent, frame: &mut Frame) {
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
                " ===================== D I S O L V =====================\n\
              \n\
              Scenario: {}\n\
              \n\
              =============================================================
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
        Active Agents:{}
        ",
            content.metadata.input_file,
            content.metadata.output_path,
            content.metadata.log_path,
            content.active_agents,
        );
        frame.render_widget(
            Paragraph::new(simulation_details)
                .block(Block::default().borders(Borders::ALL).title("More details"))
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .alignment(Alignment::Left),
            layout[2],
        );
    }
}
