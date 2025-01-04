use std::path::Path;
pub use std::path::PathBuf;

use hashbrown::HashMap;
use log::debug;

use disolv_core::agent::AgentKind;
use disolv_core::bucket::TimeMS;
use disolv_output::logger::initiate_logger;
use disolv_output::result::ResultWriter;
use disolv_output::ui::SimUIMetadata;

use crate::links::linker::{LinkerImpl, LinkType};
use crate::links::reader::{Reader, TraceType};
use crate::simulation::config::Config;

pub(crate) struct LinkFinder {
    pub(crate) step_size: TimeMS,
    pub(crate) start: TimeMS,
    pub(crate) end: TimeMS,
    config_path: PathBuf,
    config: Config,
    readers: HashMap<AgentKind, Reader>,
    linkers: Vec<LinkerImpl>,
}

impl LinkFinder {
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

    pub(crate) fn build_link_metadata(&self) -> SimUIMetadata {
        SimUIMetadata {
            scenario: "link_generation".to_string(),
            input_file: self
                .config
                .position_files
                .first()
                .unwrap()
                .position_file
                .to_string(),
            output_path: self.config.settings.output_path.to_string(),
            log_path: self.config.log_settings.log_path.clone(),
        }
    }

    pub(crate) fn initialize(&mut self) {
        initiate_logger(&self.config_path, &self.config.log_settings, None);
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

            if link_setting.link_count.is_some() && link_setting.link_radius.is_some() {
                panic!("Only one of the parameters should be given");
            }
        }

        // Initialize link finders.
        let output_path = Path::new(&self.config.settings.output_path);
        for linker_setting in self.config.link_settings.iter() {
            self.linkers
                .push(LinkerImpl::new(output_path, linker_setting));
        }

        // Read positions of devices with constant traces.
        self.readers.values_mut().for_each(|reader| {
            reader.initialize();
        });
    }

    pub(crate) fn find_links_at(&mut self, step: TimeMS) {
        self.readers.values_mut().for_each(|reader| {
            reader.update_positions_at(step);
        });

        for (link_setting, linker) in self
            .config
            .link_settings
            .iter()
            .zip(self.linkers.iter_mut())
        {
            // Skip calculating static links after 0th time step.
            if link_setting.link_type == LinkType::Static && step > self.start {
                continue;
            }

            let source_positions = match self
                .readers
                .get(&link_setting.source)
                .expect("missing reader for device type")
                .read_positions_at(step)
            {
                Some(pos) => pos,
                None => continue,
            };

            let destination_tree = self
                .readers
                .get(&link_setting.target)
                .expect("missing reader for device type")
                .get_kd_tree();

            debug!("Calculating links for {}", step);
            linker.calculate_links(source_positions, destination_tree, step);
            linker.write_to_file();
        }
    }

    pub(crate) fn complete(mut self) {
        &mut self
            .linkers
            .iter_mut()
            .for_each(|linker| linker.flush_cache());
        self.linkers
            .into_iter()
            .for_each(|linker| linker.close_file());
    }
}
