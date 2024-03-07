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

#[derive(Debug, Clone, Default)]
pub struct LinkUIMetadata {
    pub input_file: String,
    pub output_path: String,
}

#[derive(Debug, Default)]
pub struct LinkContent {
    pub running: bool,
    pub total_steps: u64,
    pub now: u64,
    pub metadata: LinkUIMetadata,
}

impl LinkContent {
    pub fn new(total_steps: u64, metadata: LinkUIMetadata) -> Self {
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
