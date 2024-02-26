use std::error;

/// Application result type.
pub type ContentResult<T> = Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug, Default)]
pub struct Content {
    pub running: bool,
    pub total_steps: u64,
    pub now: u64,
    pub total_agents: usize,
    pub active_agents: usize,
}

impl Content {
    pub fn new(total_steps: u64) -> Self {
        Self {
            total_steps,
            running: true,
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
}
