use std::path::PathBuf;

use typed_builder::TypedBuilder;

use disolv_output::result::{BasicResults, OutputSettings, OutputType, ResultWriter};

use crate::out::net::NetStatWriter;
use crate::out::tx::TxDataWriter;

#[derive(TypedBuilder)]
pub struct OutputWriter {
    pub basic_results: BasicResults,
    pub tx_data_writer: Option<TxDataWriter>,
    pub network_writer: Option<NetStatWriter>,
}

impl OutputWriter {
    pub fn new(output_config: &OutputSettings) -> Self {
        let basic_results = BasicResults::new(&output_config);
        let output_path = PathBuf::from(&output_config.output_path);
        let tx_data_writer = match output_config
            .outputs
            .iter()
            .filter(|output| output.output_type == OutputType::TxData)
            .last()
        {
            Some(settings) => Some(TxDataWriter::new(
                &output_path.join(&settings.output_filename),
            )),
            None => None,
        };
        let network_writer = match output_config
            .outputs
            .iter()
            .filter(|output| output.output_type == OutputType::NetStat)
            .last()
        {
            Some(settings) => Some(NetStatWriter::new(
                &output_path.join(&settings.output_filename),
            )),
            None => None,
        };
        Self {
            basic_results,
            tx_data_writer,
            network_writer,
        }
    }

    pub(crate) fn write_to_file(&mut self) {
        self.basic_results.positions.write_to_file();
        self.basic_results.rx_counts.write_to_file();
        if let Some(tx) = &mut self.tx_data_writer {
            tx.write_to_file();
        }
        if let Some(net) = &mut self.network_writer {
            net.write_to_file();
        }
    }

    pub fn close_output_files(mut self) {
        self.basic_results.close_files();
        if let Some(tx) = self.tx_data_writer {
            tx.close_file();
        }
        if let Some(net) = self.network_writer {
            net.close_file();
        }
    }
}
