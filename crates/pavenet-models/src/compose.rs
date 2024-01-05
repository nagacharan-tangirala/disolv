use crate::model::{Model, ModelSettings};
use log::{debug, error};
use pavenet_core::entity::NodeClass;
use pavenet_core::message::{DPayload, DataBlob, DataSource, NodeContent, PayloadInfo};
use pavenet_core::radio::{ActionImpl, DLink};
use pavenet_engine::bucket::TimeMS;
use pavenet_engine::uuid;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct ComposerSettings {
    pub name: String,
    pub source_settings: Vec<DataSource>,
}

impl ModelSettings for ComposerSettings {}

#[derive(Clone, Debug)]
pub enum Composer {
    Basic(BasicComposer),
    Status(StatusComposer),
}

impl Model for Composer {
    type Settings = ComposerSettings;

    fn with_settings(settings: &ComposerSettings) -> Self {
        match settings.name.to_lowercase().as_str() {
            "basic" => Composer::Basic(BasicComposer::new(settings)),
            "status" => Composer::Status(StatusComposer::new(settings)),
            _ => {
                error!("Only Basic and Status composers are supported.");
                panic!("Unsupported composer type {}.", settings.name);
            }
        }
    }
}

impl Composer {
    pub fn compose_payload(&self, target_class: &NodeClass, content: NodeContent) -> DPayload {
        match self {
            Composer::Basic(composer) => composer.compose_payload(target_class, content),
            Composer::Status(composer) => composer.compose_payload(target_class, content),
        }
    }

    pub fn update_sources(&mut self, data_sources: &Vec<DataSource>) {
        match self {
            Composer::Basic(composer) => composer.update_sources(data_sources),
            Composer::Status(_) => (),
        }
    }

    pub fn append_blobs_to(&mut self, payload: &mut DPayload, blobs: &mut Vec<DataBlob>) {
        blobs.iter().for_each(|blob| {
            payload.metadata.total_size += blob.data_size;
            payload.metadata.total_count += 1;
        });
        payload.metadata.data_blobs.append(blobs);
    }
}

#[derive(Clone, Debug, Default)]
pub struct BasicComposer {
    pub data_sources: Vec<DataSource>,
    pub step: TimeMS,
}

impl BasicComposer {
    pub fn new(composer_settings: &ComposerSettings) -> Self {
        Self {
            data_sources: composer_settings.source_settings.to_owned(),
            step: TimeMS::default(),
        }
    }

    pub fn update_sources(&mut self, data_sources: &Vec<DataSource>) {
        self.data_sources = data_sources.to_owned();
    }

    pub fn update_step(&mut self, step: TimeMS) {
        self.step = step;
    }

    fn compose_payload(&self, target_class: &NodeClass, content: NodeContent) -> DPayload {
        let payload_info = self.compose_metadata(target_class);
        DPayload::builder()
            .metadata(payload_info)
            .node_state(content)
            .gathered_states(Some(Vec::new()))
            .build()
    }

    fn compose_metadata(&self, target_class: &NodeClass) -> PayloadInfo {
        let mut data_blobs = Vec::with_capacity(self.data_sources.len());
        let mut data_count: u32 = 0;
        for ds_settings in self.data_sources.iter() {
            if ds_settings.node_class != *target_class {
                continue;
            }
            if self.step.as_u32() % ds_settings.source_step.as_u32() != TimeMS::default().as_u32() {
                continue;
            }

            let data_blob = DataBlob::builder()
                .data_type(ds_settings.data_type)
                .data_size(ds_settings.data_size)
                .action(ActionImpl::default())
                .build();
            data_blobs.push(data_blob);
            data_count += 1;
        }
        let payload_info = PayloadInfo::builder()
            .id(uuid::Uuid::new_v4())
            .total_size(data_blobs.iter().map(|x| x.data_size).sum())
            .data_blobs(data_blobs)
            .total_count(data_count)
            .selected_link(DLink::default())
            .build();
        debug!("Created payload with id {}", payload_info.id);
        payload_info
    }
}

#[derive(Clone, Debug)]
pub struct StatusComposer {
    pub data_sources: Vec<DataSource>,
}

impl StatusComposer {
    pub fn new(composer_settings: &ComposerSettings) -> Self {
        Self {
            data_sources: composer_settings.source_settings.to_owned(),
        }
    }

    fn compose_payload(&self, _target_class: &NodeClass, content: NodeContent) -> DPayload {
        DPayload::builder()
            .metadata(PayloadInfo::default())
            .node_state(content)
            .gathered_states(Some(Vec::new()))
            .build()
    }
}
