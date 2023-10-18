use pavenet_recipe::node_info::kind::NodeType;
use pavenet_recipe::payload::{PayloadContent, PayloadStats, TPayload, TPayloadData};
use pavenet_recipe::response::DataSource;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct DataHandler {
    pub compression: f32,
    pub sampling_factor: f32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ComposerSettings {
    pub name: String,
    pub source_settings: Vec<DataSource>,
    pub data_handler: DataHandler,
}

#[derive(Clone, Debug, Copy)]
pub enum Composer {
    Basic(BasicComposer),
    Status(StatusComposer),
}

impl Composer {
    pub fn compose_payload(
        &self,
        target_type: &NodeType,
        content: &PayloadContent,
        gathered_payloads: Option<Vec<TPayload>>,
    ) -> TPayload {
        match self {
            Composer::Basic(composer) => {
                composer.compose_payload(target_type, content, gathered_payloads)
            }
            Composer::Status(composer) => {
                composer.compose_payload(target_type, content, gathered_payloads)
            }
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct BasicComposer {
    pub data_sources: Vec<DataSource>,
    pub ds_count: usize,
    pub data_handler: DataHandler,
}

impl BasicComposer {
    pub fn new(composer_settings: &ComposerSettings) -> Self {
        Self {
            data_sources: composer_settings.ds_array(),
            ds_count: composer_settings.ds_count(),
            data_handler: composer_settings.data_handler,
        }
    }

    fn update_sources(&mut self, data_sources: Vec<DataSource>) {
        self.data_sources = data_sources;
        self.ds_count = self.data_sources.iter().map(|x| x.is_some()).len();
    }

    fn compose_metadata(&self, target_type: &NodeType) -> PayloadStats {
        let mut payload_stats = PayloadStats::default();
        for idx in 0..self.ds_count {
            let ds_settings = self.data_sources[idx].unwrap();
            if ds_settings.target_type != *target_type {
                continue;
            }

            let data_type = ds_settings.data_type;
            let mut data_counts = ds_settings.data_count;
            let mut data_size = ds_settings.unit_size * data_counts as f32;

            data_counts = data_counts * self.data_handler.sampling_factor;
            data_size = data_size * self.data_handler.compression;

            payload_stats.size_by_type.insert(data_type, data_size);
            payload_stats.count_by_type.insert(data_type, data_counts);
        }
        return payload_stats;
    }

    fn compose_payload(
        &self,
        target_type: &NodeType,
        content: &PayloadContent,
        gathered_payloads: Option<Vec<TPayload>>,
    ) -> TPayload {
        let payload_data = TPayloadData::new(*content, content.node_info.id);
        let mut payload_stats = self.compose_metadata(target_type);
        let mut in_content = None;

        match gathered_payloads {
            Some(data) => {
                self.capture_incoming_stats(&mut payload_stats, &data);
                in_content = self.extract_content(data);
            }
            None => {}
        };

        let payload = TPayload::new(payload_data, payload_stats, in_content);
        return payload;
    }

    fn capture_incoming_stats(&self, payload_stats: &mut PayloadStats, incoming: &Vec<TPayload>) {
        let in_stats: Vec<PayloadStats> = incoming.iter().map(|p| p.payload_stats).collect();
        for in_stat in in_stats.into_iter() {
            for (d_type, count) in in_stat.count_by_type {
                payload_stats.count_by_type.entry(d_type).or_insert(count);
            }
            for (d_type, count) in in_stat.size_by_type {
                payload_stats.size_by_type.entry(d_type).or_insert(count);
            }
            payload_stats.total_count += in_stat.total_count;
            payload_stats.total_size += in_stat.total_size;
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct StatusComposer {
    pub data_sources: Vec<DataSource>,
    pub ds_count: usize,
    pub data_handler: DataHandler,
}

impl StatusComposer {
    pub fn new(composer_settings: &ComposerSettings) -> Self {
        Self {
            data_sources: composer_settings.ds_array(),
            ds_count: composer_settings.ds_count(),
            data_handler: composer_settings.data_handler,
        }
    }
}
