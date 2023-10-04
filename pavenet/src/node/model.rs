use pavenet_config::config::base::{ComposerSettings, SimplifierSettings};
use pavenet_config::config::dynamic::EpisodeInfo;

#[derive(Debug, Copy, Clone)]
pub struct DeviceModel {
    pub power_schedule: PowerSchedule,
    pub composer: Option<ComposerType>,
    pub simplifier: Option<SimplifierType>,
    pub linker: Option<LinkerType>,
}

impl DeviceModel {
    pub fn new() -> ModelBuilder {
        ModelBuilder::new()
    }

    fn update_models(&mut self, episode: &EpisodeInfo, reset_ts: TimeStamp) {
        if let Some(data_sources) = &episode.data_sources {
            match &mut self.composer {
                Some(ComposerType::Basic(ref mut composer)) => {
                    composer.update_settings(data_sources);
                }
                Some(ComposerType::Status(ref mut composer)) => {
                    composer
                        .settings_handler
                        .update_settings(data_sources, reset_ts);
                }
                None => {}
            }
        }

        if let Some(simplifier_settings) = &episode.simplifier {
            match &mut self.simplifier {
                SimplifierType::Basic(ref mut simplifier) => {
                    simplifier
                        .settings
                        .update_settings(simplifier_settings, reset_ts);
                }
            }
        }

        if let Some(linker_settings) = &episode.veh_linker {
            match &mut self.linker {
                VehLinkerType::Simple(ref mut linker) => {
                    linker
                        .settings_handler
                        .update_settings(linker_settings, reset_ts);
                }
            }
        }
    }
}

pub struct ModelBuilder {
    pub power_schedule: PowerSchedule,
    pub composer: Option<ComposerType>,
    pub simplifier: Option<SimplifierType>,
    pub linker: Option<LinkerType>,
}

impl ModelBuilder {
    pub fn new() -> Self {
        Self {
            power_schedule: PowerSchedule::default(),
            composer: None,
            simplifier: None,
            linker: None,
        }
    }

    pub fn with_power_schedule(mut self, power_schedule: PowerSchedule) -> Self {
        self.power_schedule = power_schedule;
        self
    }

    pub fn with_composer(mut self, composer_settings: &Option<ComposerSettings>) -> Self {
        self.composer = match composer_settings {
            Some(ref composer_settings) => match composer_settings.name.as_str() {
                "basic" => Some(ComposerType::Basic(BasicComposer::new(composer_settings))),
                "status" => Some(ComposerType::Status(StatusComposer::new(composer_settings))),
                _ => panic!("Unknown composer type"),
            },
            None => None,
        };
        self
    }

    pub fn with_simplifier(mut self, simplifier: &Option<SimplifierSettings>) -> Self {
        self.simplifier = match simplifier {
            Some(ref simplifier_settings) => match simplifier_settings.name.as_str() {
                "basic" => Some(SimplifierType::Basic(BasicSimplifier::new(
                    simplifier_settings,
                ))),
                "random" => Some(SimplifierType::Random(RandomSimplifier::new(
                    simplifier_settings,
                ))),
                _ => panic!("Unknown simplifier type"),
            },
            None => None,
        };
        self
    }

    pub fn with_linker(mut self, linker: LinkerType) -> Self {
        self.linker = Some(linker);
        self
    }

    pub fn build(self) -> DeviceModel {
        DeviceModel {
            power_schedule: self.power_schedule,
            composer: self.composer,
            simplifier: self.simplifier,
            linker: self.linker,
        }
    }
}
