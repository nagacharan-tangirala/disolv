use std::path::PathBuf;

use hashbrown::HashMap;

use disolv_core::bucket::TimeMS;
use disolv_models::device::types::DeviceType;

use crate::config::Config;
use crate::linker::{LinkerImpl, LinkType};
use crate::logger;
use crate::reader::{Reader, TraceType};
use crate::ui::LinkUIMetadata;

pub(crate) struct LinkBuilder {
    pub(crate) step_size: TimeMS,
    pub(crate) start: TimeMS,
    pub(crate) end: TimeMS,
    config_path: PathBuf,
    config: Config,
    readers: HashMap<DeviceType, Reader>,
    linkers: Vec<LinkerImpl>,
}

impl LinkBuilder {
    pub(crate) fn new(config: Config, config_path: PathBuf) -> Self {
        Self {
            start: config.settings.start,
            end: config.settings.end,
            step_size: config.settings.step_size,
            readers: HashMap::with_capacity(config.position_files.len()),
            linkers: Vec::with_capacity(config.link_settings.len()),
            config,
            config_path,
        }
    }

    pub(crate) fn build_link_metadata(&self) -> LinkUIMetadata {
        LinkUIMetadata {
            input_file: self
                .config_path
                .to_str()
                .expect("failed to convert")
                .to_owned(),
            output_path: self.config.settings.output_path.to_owned(),
        }
    }

    pub(crate) fn initialize(&mut self) {
        logger::initiate_logger(&self.config_path, &self.config.log_settings);
        for pos_file in self.config.position_files.iter() {
            self.readers.insert(pos_file.device, Reader::new(pos_file));
        }

        // Validate if the configuration is correct.
        for link_setting in self.config.link_settings.iter() {
            match link_setting.link_type {
                LinkType::Static => {
                    // Static links should have both the source and target traces as Constant
                    if let Some(reader) = self.readers.get(&link_setting.source) {
                        assert_eq!(reader.trace_type(), TraceType::Constant);
                    }
                    if let Some(reader) = self.readers.get(&link_setting.target) {
                        assert_eq!(reader.trace_type(), TraceType::Constant);
                    }
                }
                LinkType::Dynamic => {
                    // Dynamic links should contain at least one Mobile trace.
                    let source_reader = self
                        .readers
                        .get(&link_setting.source)
                        .expect("Invalid source device type");
                    let target_reader = self
                        .readers
                        .get(&link_setting.target)
                        .expect("Invalid target device type");
                    if source_reader.trace_type() != TraceType::Mobile
                        && target_reader.trace_type() != TraceType::Mobile
                    {
                        panic!("One of the device traces should be Mobile for dynamic links");
                    }
                }
            }
        }

        // Initialize link finders.
        for linker_setting in self.config.link_settings.iter() {
            self.linkers.push(LinkerImpl::new(
                self.config.settings.output_path.as_str(),
                linker_setting,
            ));
        }

        // Read positions of devices with constant traces.
        self.readers.values_mut().for_each(|reader| {
            reader.initialize();
        });
    }

    pub(crate) fn build_links_at(&mut self, step: TimeMS) {
        self.readers.values_mut().for_each(|reader| {
            reader.update_positions_at(step);
        });

        for (link_setting, writer) in self
            .config
            .link_settings
            .iter()
            .zip(self.linkers.iter_mut())
        {
            // Skip calculating static links after 0th time step.
            if link_setting.link_type == LinkType::Static && step > self.start {
                continue;
            }

            let positions = match self
                .readers
                .get(&link_setting.source)
                .expect("missing reader for device type")
                .read_positions_at(step)
            {
                Some(pos) => pos,
                None => continue,
            };

            let position_tree = self
                .readers
                .get(&link_setting.target)
                .expect("missing reader for device type")
                .get_kd_tree();

            writer.write_links(positions, position_tree, step);
        }
    }

    pub(crate) fn complete(self) {
        self.linkers.into_iter().for_each(|w| w.flush())
    }
}
