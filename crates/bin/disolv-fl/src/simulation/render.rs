use burn::train::renderer::{MetricsRenderer, MetricState, TrainingProgress};

pub(crate) struct CustomRenderer {}

impl MetricsRenderer for CustomRenderer {
    fn update_train(&mut self, _state: MetricState) {}

    fn update_valid(&mut self, _state: MetricState) {}

    fn render_train(&mut self, item: TrainingProgress) {
        //println!("Epoch - {}, iteration - {}", item.epoch, item.iteration);
    }

    fn render_valid(&mut self, item: TrainingProgress) {}
}
