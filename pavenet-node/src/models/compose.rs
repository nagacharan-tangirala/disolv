use pavenet_core::entity::class::NodeClass;
use pavenet_core::payload::{DPayload, NodeContent, PayloadInfo};
use pavenet_core::response::DataSource;
use pavenet_engine::bucket::TimeS;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Default, Copy)]
pub struct DataHandler {
    pub compression: f32,
    pub sampling_factor: f32,
}

impl DataHandler {
    pub fn apply(&self, payload: DPayload) -> DPayload {
        let mut payload = payload;
        payload.metadata.total_size = payload.metadata.total_size * self.compression;
        payload.metadata.total_count =
            (payload.metadata.total_count as f32 * self.sampling_factor).round() as u32;
        payload
    }
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
        target_class: &NodeClass,
        content: NodeContent,
        gathered_payloads: &Vec<DPayload>,
    ) -> DPayload {
        match self {
            Composer::Basic(composer) => {
                composer.compose_payload(target_class, content, gathered_payloads)
            }
            Composer::Status(composer) => {
                composer.compose_payload(target_class, content, gathered_payloads)
            }
        }
    }

    pub fn update_sources(&mut self, data_sources: &Vec<DataSource>) {
        match self {
            Composer::Basic(composer) => composer.update_sources(data_sources),
            Composer::Status(_) => (),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct BasicComposer {
    pub data_sources: Vec<DataSource>,
    pub data_handler: DataHandler,
    pub step: TimeS,
}

impl BasicComposer {
    pub fn new(composer_settings: ComposerSettings) -> Self {
        Self {
            data_sources: composer_settings.source_settings,
            data_handler: composer_settings.data_handler,
            step: TimeS::default(),
        }
    }

    pub fn update_sources(&mut self, data_sources: &Vec<DataSource>) {
        self.data_sources = data_sources.to_owned();
    }

    pub fn update_step(&mut self, step: TimeS) {
        self.step = step;
    }

    fn compose_payload(
        &self,
        target_class: &NodeClass,
        content: NodeContent,
        gathered_payloads: &Vec<DPayload>,
    ) -> DPayload {
        let payload_stats = self.compose_metadata(target_class);
        let content_to_fwd = match self.compose_fwd_content(target_class, gathered_payloads) {
            content if content.is_empty() => None,
            content => Some(content),
        };
        let updated_stats = self.update_metadata(payload_stats, target_class, gathered_payloads);
        let payload = DPayload::new(content, updated_stats, content_to_fwd);
        let payload = self.data_handler.apply(payload);
        return payload;
    }

    fn compose_metadata(&self, target_class: &NodeClass) -> PayloadInfo {
        let mut payload_stats = PayloadInfo::default();
        payload_stats.tx_info.next_hop = *target_class;
        for ds_settings in self.data_sources.iter() {
            if ds_settings.node_class != *target_class {
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
        let inc_classes: Vec<NodeClass> = incoming
            .iter()
            .map(|x| x.metadata.tx_info.next_hop)
            .collect();

        for (inc_payload, inc_class) in incoming.iter().zip(inc_classes.into_iter()) {
            // No need to forward if the target is not the intended one
            if *target_class != inc_class {
                continue;
            }
            for (d_type, count) in inc_payload.metadata.count_by_type.iter() {
                updated_info
                    .count_by_type
                    .entry(*d_type)
                    .and_modify(|c| *c += count)
                    .or_insert(*count);
            }
            for (d_type, count) in inc_payload.metadata.size_by_type.iter() {
                updated_info
                    .size_by_type
                    .entry(*d_type)
                    .and_modify(|c| *c += count)
                    .or_insert(*count);
            }
            updated_info.total_count += inc_payload.metadata.total_count;
            updated_info.total_size += inc_payload.metadata.total_size;
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
        target_class: &NodeClass,
        content: NodeContent,
        gathered_payloads: &Vec<DPayload>,
    ) -> DPayload {
        DPayload::new(
            content,
            PayloadInfo::default(),
            Some(self.compose_fwd_content(target_class, &gathered_payloads)),
        )
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
