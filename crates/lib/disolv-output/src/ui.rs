use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::Frame;

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

pub trait Renderer: Send {
    fn render_sim_ui(&self, content: &mut SimContent, frame: &mut Frame);
}
