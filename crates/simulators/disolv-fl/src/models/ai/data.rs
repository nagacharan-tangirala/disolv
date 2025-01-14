use serde::Deserialize;

use disolv_core::bucket::TimeMS;
use disolv_core::model::{Model, ModelSettings};

use crate::models::ai::models::DatasetType;

#[derive(Clone, Debug, Deserialize)]
pub struct DataStrategySettings {
    pub variant: String,
    #[serde(default)]
    pub steps_per_unit: Option<TimeMS>,
    pub to_clone: bool,
}

impl ModelSettings for DataStrategySettings {}

#[derive(Clone)]
pub enum DataStrategy {
    Time(TimeStrategy),
    Location(LocationStrategy),
    All(AllStrategy),
}

impl Model for DataStrategy {
    type Settings = DataStrategySettings;

    fn with_settings(settings: &Self::Settings) -> Self {
        match settings.variant.to_lowercase().as_str() {
            "time" => DataStrategy::Time(TimeStrategy::new(settings)),
            "location" => DataStrategy::Location(LocationStrategy::new(settings)),
            "all" => DataStrategy::All(AllStrategy::new(settings)),
            _ => panic!("Invalid data strategy. Only time and location are supported"),
        }
    }
}

impl DataStrategy {
    pub fn allot_data(
        &mut self,
        allotted_data: &mut DatasetType,
        total_data: &mut DatasetType,
        step: TimeMS,
    ) {
        match self {
            DataStrategy::Time(time_strat) => {
                time_strat.allot_data(allotted_data, total_data, step)
            }
            DataStrategy::All(all_strat) => all_strat.allot_data(allotted_data, total_data),
            DataStrategy::Location(_) => unimplemented!("location pending"),
        }
    }
}

#[derive(Clone)]
pub struct TimeStrategy {
    pub(crate) steps_per_unit: TimeMS,
    pub(crate) to_clone: bool,
}

impl TimeStrategy {
    pub fn new(settings: &DataStrategySettings) -> Self {
        Self {
            steps_per_unit: settings.steps_per_unit.expect("Units per step is missing"),
            to_clone: settings.to_clone,
        }
    }

    pub fn allot_data(
        &mut self,
        allotted_data: &mut DatasetType,
        total_data: &mut DatasetType,
        step: TimeMS,
    ) {
        if step.as_u64() % self.steps_per_unit.as_u64() != 0 {
            return;
        }
        match total_data {
            DatasetType::Mnist(mnist) => {
                let images_to_move = 1;
                if self.to_clone {
                    let mut dataset = mnist.clone();
                    for _ in 0..images_to_move {
                        let mnist_image = dataset.images.pop().expect("failed to read image");
                        allotted_data.append_mnist(mnist_image);
                    }
                } else {
                    for _ in 0..images_to_move {
                        let mnist_image = mnist.images.pop().expect("failed to read image");
                        allotted_data.append_mnist(mnist_image);
                    }
                }
            }
            _ => unimplemented!("only mnist is valid"),
        }
    }
}

#[derive(Clone)]
pub struct AllStrategy {
    to_clone: bool,
}

impl AllStrategy {
    pub fn new(settings: &DataStrategySettings) -> Self {
        Self {
            to_clone: settings.to_clone,
        }
    }

    pub fn allot_data(&self, allotted_data: &mut DatasetType, total_data: &mut DatasetType) {
        match total_data {
            DatasetType::Mnist(mnist) => {
                mnist
                    .images
                    .iter()
                    .for_each(|item| allotted_data.append_mnist(item.to_owned()));
            }
            _ => unimplemented!("only mnist is valid"),
        }
        if !self.to_clone {
            total_data.clear();
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
    pub fn set_train_data(&mut self, dataset: DatasetType) {
        self.allotted_train = DatasetType::blank(dataset.dataset_type());
        self.usable_train = dataset;
    }

    pub fn set_test_data(&mut self, dataset: DatasetType) {
        self.allotted_test = DatasetType::blank(dataset.dataset_type());
        self.usable_test = dataset;
    }

    pub fn allot_data(&mut self, step: TimeMS) {
        if self.usable_train.has_data() {
            self.strategy
                .allot_data(&mut self.allotted_train, &mut self.usable_train, step);
        }
        if self.usable_test.has_data() {
            self.strategy
                .allot_data(&mut self.allotted_test, &mut self.usable_test, step);
        }
    }

    pub fn allotted_train_data(&mut self) -> DatasetType {
        let dataset_type = self.allotted_train.dataset_type().to_owned();
        std::mem::replace(&mut self.allotted_train, DatasetType::blank(&dataset_type))
    }

    pub fn allotted_test_data(&mut self) -> DatasetType {
        let dataset_type = self.allotted_test.dataset_type().to_owned();
        std::mem::replace(&mut self.allotted_test, DatasetType::blank(&dataset_type))
    }
}
