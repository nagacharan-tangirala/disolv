use std::error;

/// Application result type.
pub type ContentResult<T> = Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone, Default)]
pub struct SimulationMetadata {
    pub scenario: String,
    pub input_file: String,
    pub output_path: String,
    pub log_path: String,
}

/// Application.
#[derive(Debug, Default)]
pub struct Content {
    pub running: bool,
    pub total_steps: u64,
    pub now: u64,
    pub metadata: SimulationMetadata,
    pub total_agents: usize,
    pub active_agents: usize,
}

impl Content {
    pub fn new(total_steps: u64, metadata: SimulationMetadata) -> Self {
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
