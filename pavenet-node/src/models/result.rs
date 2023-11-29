use pavenet_core::result::OutputSettings;
use pavenet_engine::bucket::TimeS;
use pavenet_output::result::ResultWriter;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Resultant {
    pub result_writer: ResultWriter,
}

impl Resultant {
    pub fn new(output_path: &PathBuf, sim_step: TimeS, output_settings: &OutputSettings) -> Self {
        let result_writer = ResultWriter::builder(output_path, sim_step, &output_settings)
            .with_node_pos_writer()
            .with_tx_writer()
            .with_rx_writer()
            .build();

        Self { result_writer }
    }
}
