use crate::config::Config;
use crate::finder::LinkFinder;
use crate::linker::LinkType;
use crate::logger;
use crate::reader::{Reader, TraceType};
use disolv_core::bucket::TimeMS;
use disolv_models::device::types::DeviceType;
use hashbrown::HashMap;
use std::path::PathBuf;

pub(crate) struct LinkBuilder {
    step_size: TimeMS,
    now: TimeMS,
    start: TimeMS,
    end: TimeMS,
    config_path: PathBuf,
    config: Config,
    readers: HashMap<DeviceType, Reader>,
}

impl LinkBuilder {
    pub(crate) fn new(config: Config, config_path: PathBuf) -> Self {
        Self {
            start: config.settings.start,
            end: config.settings.end,
            step_size: config.settings.step_size,
            readers: HashMap::with_capacity(config.position_files.len()),
            now: TimeMS::default(),
            config,
            config_path,
        }
    }

    pub(crate) fn initiate(&mut self) {
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
                        assert_eq!(reader.trace_type, TraceType::Constant);
                    }
                    if let Some(reader) = self.readers.get(&link_setting.target) {
                        assert_eq!(reader.trace_type, TraceType::Constant);
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
                    if source_reader.trace_type != TraceType::Mobile
                        && target_reader.trace_type != TraceType::Mobile
                    {
                        panic!("One of the device traces should be Mobile for dynamic links");
                    }
                }
            }
        }
    }

    pub(crate) fn build_links(&mut self) {
        let mut writers = Vec::with_capacity(self.config.link_settings.len());
        for linker_setting in self.config.link_settings.iter() {
            writers.push(LinkFinder::new(
                self.config.settings.output_path.as_str(),
                linker_setting,
            ));
        }

        // Read constant positions before beginning the loop.
        self.readers.values_mut().for_each(|reader| {
            if reader.trace_type == TraceType::Constant {
                reader.read_constant_positions();
            }
        });

        self.now = self.start;
        while self.now < self.end {
            self.readers.values_mut().for_each(|reader| {
                if reader.trace_type == TraceType::Mobile {
                    reader.read_dynamic_positions_at(self.now);
                }
            });

            for (link_setting, writer) in self.config.link_settings.iter().zip(writers.iter_mut()) {
                // Skip calculating static links after 0th time step.
                if link_setting.link_type == LinkType::Static && self.now > self.start {
                    continue;
                }

                let positions = match self
                    .readers
                    .get(&link_setting.source)
                    .expect("missing reader for device type")
                    .get_positions_at(self.now)
                {
                    Some(pos) => pos,
                    None => continue,
                };

                let position_tree = self
                    .readers
                    .get(&link_setting.target)
                    .expect("missing reader for device type")
                    .get_position_tree();
                writer.write_links(positions, position_tree, self.now);
            }
            self.now += self.step_size;
        }
        writers.into_iter().for_each(|w| w.flush())
    }
}
