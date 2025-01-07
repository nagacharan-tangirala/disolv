use burn::train::renderer::{MetricState, MetricsRenderer, TrainingProgress};
use log::debug;

pub(crate) struct CustomRenderer {}

impl MetricsRenderer for CustomRenderer {
    fn update_train(&mut self, _state: MetricState) {}

    fn update_valid(&mut self, _state: MetricState) {}

    fn render_train(&mut self, item: TrainingProgress) {
        debug!("Epoch - {}, iteration - {}", item.epoch, item.iteration);
    }

    fn render_valid(&mut self, item: TrainingProgress) {}
}
