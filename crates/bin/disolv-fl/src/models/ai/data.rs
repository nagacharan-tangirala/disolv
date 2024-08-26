use std::cmp::min;

use serde::Deserialize;

use disolv_core::model::{Model, ModelSettings};

use crate::models::ai::models::DatasetType;

#[derive(Clone, Debug, Deserialize)]
pub struct DataStrategySettings {
    pub variant: String,
    pub units_per_step: usize,
    pub test_train_split: f64,
}

impl ModelSettings for DataStrategySettings {}

#[derive(Clone)]
pub enum DataStrategy {
    Time(TimeStrategy),
    Location(LocationStrategy),
}

impl Model for DataStrategy {
    type Settings = DataStrategySettings;

    fn with_settings(settings: &Self::Settings) -> Self {
        match settings.variant.to_lowercase().as_str() {
            "time" => DataStrategy::Time(TimeStrategy::new(settings)),
            "location" => DataStrategy::Location(LocationStrategy::new(settings)),
            _ => panic!("Invalid data strategy. Only time and location are supported"),
        }
    }
}

impl DataStrategy {
    pub fn allot_training_data(
        &self,
        allotted_data: &mut DatasetType,
        total_data: &mut DatasetType,
    ) {
        match self {
            DataStrategy::Time(time_strat) => {
                time_strat.allot_training_data(allotted_data, total_data)
            }
            DataStrategy::Location(loc_strat) => unimplemented!("location pending"),
        }
    }
}

#[derive(Clone)]
pub struct TimeStrategy {
    pub(crate) units_per_step: usize,
}

impl TimeStrategy {
    pub fn new(settings: &DataStrategySettings) -> Self {
        Self {
            units_per_step: settings.units_per_step,
        }
    }

    pub fn allot_training_data(
        &self,
        allotted_data: &mut DatasetType,
        total_data: &mut DatasetType,
    ) {
        match total_data {
            DatasetType::Mnist(mnist) => {
                let images_to_move = min(mnist.images.len(), self.units_per_step);
                for i in 0..images_to_move {
                    allotted_data.append_mnist(mnist.images.pop().expect("failed to read image"));
                }
            }
            _ => unimplemented!("only mnist is valid"),
        }
    }
}

#[derive(Clone)]
pub struct LocationStrategy {}

impl LocationStrategy {
    pub fn new(_settings: &DataStrategySettings) -> Self {
        unimplemented!("location strategy not implemented");
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct DataHolderSettings {
    strategy: DataStrategySettings,
}

impl ModelSettings for DataHolderSettings {}

#[derive(Clone)]
pub struct DataHolder {
    allotted_test: DatasetType,
    allotted_train: DatasetType,
    usable_test: DatasetType,
    usable_train: DatasetType,
    strategy: DataStrategy,
}

impl Model for DataHolder {
    type Settings = DataHolderSettings;

    fn with_settings(settings: &Self::Settings) -> Self {
        let strategy = DataStrategy::with_settings(&settings.strategy);
        Self {
            allotted_test: DatasetType::Empty,
            allotted_train: DatasetType::Empty,
            usable_test: DatasetType::Empty,
            usable_train: DatasetType::Empty,
            strategy,
        }
    }
}

impl DataHolder {
    pub fn allot_data(&mut self) {
        if self.usable_train.has_data() {
            self.strategy
                .allot_training_data(&mut self.allotted_train, &mut self.usable_train);
        }
        if self.usable_test.has_data() {
            self.strategy
                .allot_training_data(&mut self.allotted_test, &mut self.usable_test);
        }
    }
}
