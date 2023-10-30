use pavenet_core::entity::class::NodeClass;
use pavenet_core::entity::kind::NodeType;
use pavenet_core::payload::{DPayload, NodeContent, PayloadInfo, PayloadTxInfo};
use pavenet_core::response::DataSource;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Default, Copy)]
pub struct DataHandler {
    pub compression: f32,
    pub sampling_factor: f32,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct ComposerSettings {
    pub name: String,
    pub source_settings: Vec<DataSource>,
    pub data_handler: DataHandler,
}

#[derive(Clone, Debug)]
pub enum Composer {
    Basic(BasicComposer),
    Status(StatusComposer),
}

impl Composer {
    pub fn compose_payload(
        &self,
        target_type: &NodeClass,
        content: NodeContent,
        gathered_payloads: &Vec<DPayload>,
    ) -> DPayload {
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

#[derive(Clone, Debug, Default)]
pub struct BasicComposer {
    pub data_sources: Vec<DataSource>,
    pub data_handler: DataHandler,
}

impl BasicComposer {
    pub fn new(composer_settings: ComposerSettings) -> Self {
        Self {
            data_sources: composer_settings.source_settings,
            data_handler: composer_settings.data_handler,
        }
    }

    fn update_sources(&mut self, data_sources: Vec<DataSource>) {
        self.data_sources = data_sources;
    }

    fn compose_payload(
        &self,
        target_class: &NodeClass,
        content: NodeContent,
        gathered_payloads: &Vec<DPayload>,
    ) -> DPayload {
        let payload_stats = self.compose_metadata(target_class);
        let updated_stats = self.update_metadata(payload_stats, target_class, gathered_payloads);
        let content_to_fwd = match self.compose_fwd_content(target_class, gathered_payloads) {
            content if content.is_empty() => None,
            content => Some(content),
        };
        let payload = DPayload::new(content, updated_stats, content_to_fwd);
        return payload;
    }

    fn compose_metadata(&self, target_type: &NodeClass) -> PayloadInfo {
        let mut payload_stats = PayloadInfo::default();
        for ds_settings in self.data_sources.iter() {
            if ds_settings.final_target != *target_type {
                continue;
            }

            let data_type = ds_settings.data_type;
            let mut data_counts = ds_settings.data_count;
            let mut data_size = ds_settings.unit_size * data_counts as f32;

            data_counts = (data_counts as f32 * self.data_handler.sampling_factor).round() as u32;
            data_size = data_size * self.data_handler.compression;

            payload_stats.size_by_type.insert(data_type, data_size);
            payload_stats.count_by_type.insert(data_type, data_counts);
        }
        return payload_stats;
    }

    fn update_metadata(
        &self,
        payload_stats: PayloadInfo,
        target_class: &NodeClass,
        incoming: &Vec<DPayload>,
    ) -> PayloadInfo {
        let mut updated_info = payload_stats.clone();
        let inc_metadata: Vec<PayloadInfo> = incoming.iter().map(|x| x.metadata).collect();
        let inc_tx_info: Vec<PayloadTxInfo> = incoming.iter().map(|x| x.metadata.tx_info).collect();

        for (inc_stat, inc_tx_info) in inc_metadata.into_iter().zip(inc_tx_info.into_iter()) {
            // No need to forward if the target is not the intended one
            if target_class != inc_tx_info.next_hop {
                continue;
            }
            for (d_type, count) in inc_stat.count_by_type {
                updated_info
                    .count_by_type
                    .entry(d_type)
                    .and_modify(|c| *c += count)
                    .or_insert(count);
            }
            for (d_type, count) in inc_stat.size_by_type {
                updated_info
                    .size_by_type
                    .entry(d_type)
                    .and_modify(|c| *c += count)
                    .or_insert(count);
            }
            updated_info.total_count += inc_stat.total_count;
            updated_info.total_size += inc_stat.total_size;
        }
        return updated_info;
    }

    fn compose_fwd_content(
        &self,
        target_class: &NodeClass,
        gathered_payloads: &Vec<DPayload>,
    ) -> Vec<NodeContent> {
        let mut fwd_content = Vec::new();
        for payload in gathered_payloads.iter() {
            if payload.metadata.tx_info.next_hop != *target_class {
                continue;
            }
            fwd_content.push(payload.content);
        }
        return fwd_content;
    }
}

#[derive(Clone, Debug)]
pub struct StatusComposer {
    pub data_sources: Vec<DataSource>,
    pub data_handler: DataHandler,
}

impl StatusComposer {
    pub fn new(composer_settings: ComposerSettings) -> Self {
        Self {
            data_sources: composer_settings.source_settings,
            data_handler: composer_settings.data_handler,
        }
    }

    fn compose_payload(
        &self,
        target_type: &NodeClass,
        content: NodeContent,
        gathered_payloads: &Vec<DPayload>,
    ) -> DPayload {
        DPayload::default()
    }
}
