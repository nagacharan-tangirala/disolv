use crate::payload::{DataType, PayloadInfo};
use crate::radio::metrics::latency::Latency;
use pavenet_engine::radio::{IncomingStats, Metric, OutgoingStats};

#[derive(Default, Clone, Copy, Debug)]
pub struct Counts {
    pub node_count: u32,
    pub data_size: f32,
    pub data_count: u32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct InDataStats {
    pub attempted: Counts,
    pub feasible: Counts,
    pub avg_latency: Latency,
}

impl InDataStats {
    pub fn new() -> Self {
        InDataStats::default()
    }

    pub fn update_latency(&mut self, latencies: Vec<Latency>) {
        let mut total = Latency::default();
        for latency in latencies.iter() {
            total += *latency;
        }
        self.avg_latency = Latency::from(total.as_f32() / latencies.len() as f32);
    }
}

impl IncomingStats<PayloadInfo, DataType> for InDataStats {
    fn add_attempted(&mut self, metadata: &PayloadInfo) {
        self.attempted.node_count += 1;
        self.attempted.data_size += metadata.total_size;
        self.attempted.data_count += metadata.total_count;
    }

    fn add_feasible(&mut self, metadata: &PayloadInfo) {
        self.feasible.node_count += 1;
        self.feasible.data_size += metadata.total_size;
        self.feasible.data_count += metadata.total_count;
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct OutDataStats {
    pub out_counts: Counts,
}

impl OutDataStats {
    pub fn new() -> Self {
        OutDataStats::default()
    }
}

impl OutgoingStats<PayloadInfo, DataType> for OutDataStats {
    fn update(&mut self, metadata: &PayloadInfo) {
        self.out_counts.node_count += 1;
        self.out_counts.data_size += metadata.total_size;
        self.out_counts.data_count += metadata.total_count;
    }
}
