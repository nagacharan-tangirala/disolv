use pavenet_config::config::base::SimplifierSettings;
use pavenet_config::config::types::TimeStamp;
use pavenet_core::::config::SimplifierSettings;
use crate::node::composer::Payload;
use crate::reader::activation::TimeStamp;

#[derive(Clone, Debug, Copy)]
struct Factors {
    compression_factor: Option<f32>,
    sampling_factor: Option<f32>,
}

#[derive(Clone, Debug, Copy)]
pub(crate) struct SettingsHandler {
    in_use: Factors,
    defaults: Factors,
    reset_time: TimeStamp,
}

#[derive(Clone, Debug, Copy)]
pub enum SimplifierType {
    Basic(BasicSimplifier),
    Random(RandomSimplifier),
}

#[derive(Clone, Debug, Copy)]
pub struct BasicSimplifier {
    pub settings: SettingsHandler,
}

#[derive(Clone, Debug, Copy)]
pub struct RandomSimplifier {
    pub settings: SettingsHandler,
}

impl SettingsHandler {
    pub fn new(simplifier_settings: &SimplifierSettings) -> Self {
        let factors = Factors {
            compression_factor: simplifier_settings.compression_factor,
            sampling_factor: simplifier_settings.sampling_factor,
        };
        Self {
            in_use: factors,
            defaults: factors,
            reset_time: 0,
        }
    }

    fn reset_to_default(&mut self) {
        self.in_use = self.defaults.clone();
    }

    pub fn update_settings(
        &mut self,
        simplifier_settings: &SimplifierSettings,
        reset_ts: TimeStamp,
    ) {
        if reset_ts > 0 {
            self.set_temporary_settings(simplifier_settings, reset_ts);
        } else {
            self.set_persistent_settings(simplifier_settings);
        }
    }

    fn set_temporary_settings(&mut self, simplifier: &SimplifierSettings, reset_ts: TimeStamp) {
        self.in_use.compression_factor = simplifier.compression_factor;
        self.in_use.sampling_factor = simplifier.sampling_factor;
        self.reset_time = reset_ts;
    }

    fn set_persistent_settings(&mut self, simplifier: &SimplifierSettings) {
        self.defaults.compression_factor = simplifier.compression_factor;
        self.defaults.sampling_factor = simplifier.sampling_factor;
        self.reset_time = TimeStamp::default();
    }
}

impl BasicSimplifier {
    pub fn new(simplifier_settings: &SimplifierSettings) -> Self {
        let settings_handler = SettingsHandler::new(&simplifier_settings);
        Self {
            settings: settings_handler,
        }
    }

    pub(crate) fn simplify_payload(&self, payload: Payload) -> Payload {
        let mut simplified_payload = payload.clone();
        match self.settings.in_use.compression_factor {
            Some(factor) => {
                simplified_payload.sensor_data.size_by_type = payload
                    .sensor_data
                    .size_by_type
                    .iter()
                    .map(|(sensor_type, data_size)| (*sensor_type, data_size * factor))
                    .collect();
            }
            None => {}
        }
        match self.settings.in_use.sampling_factor {
            Some(factor) => {
                simplified_payload.sensor_data.count_by_type = payload
                    .sensor_data
                    .count_by_type
                    .iter()
                    .map(|(sensor_type, data_count)| {
                        (*sensor_type, (*data_count as f32 * factor) as u32)
                    })
                    .collect();
            }
            None => {}
        }
        simplified_payload
    }
}

impl RandomSimplifier {
    pub fn new(simplifier_settings: &SimplifierSettings) -> Self {
        let settings_handler = SettingsHandler::new(&simplifier_settings);
        Self {
            settings: settings_handler,
        }
    }

    pub(crate) fn simplify_payload(&self, payload: Payload) -> Payload {
        payload
    }
}
