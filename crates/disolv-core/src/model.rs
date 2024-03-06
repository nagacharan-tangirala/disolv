use crate::bucket::TimeMS;

/// A marker trait for model settings. Use this to define the settings for a model. These
/// settings should be readable from a config file.
pub trait ModelSettings: Clone {}

/// A marker trait for models. Use this to define a model. A model is a struct that contains
/// a behaviour that can be used by the agent.
pub trait Model {
    type Settings: ModelSettings;

    fn with_settings(settings: &Self::Settings) -> Self;
}

/// A trait for models that are used by the bucket. This trait defines the behaviour of the
/// model. The bucket will call the methods of this trait to run the model.
/// Use this trait when the behaviour of the devices either by class or by kind is the same and
/// a periodic update is needed.
pub trait BucketModel {
    /// Initialize the model, called once at the beginning of the simulation.
    fn init(&mut self, step: TimeMS);

    /// Stream data required by the model, called when streaming interval is reached.
    fn stream_data(&mut self, step: TimeMS);

    /// Prepare the model before the agents are stepped.
    fn before_agent_step(&mut self, step: TimeMS);
}
