use crate::bucket::TimeS;
use serde::Serialize;
use std::fmt::Debug;

/// The `Saveable` trait defines the methods that take the simulator data and prepare the
/// data for plotting. This can be optionally implemented to a struct that implements the bucket
/// trait so that there is a single place to calculate the data for plotting or writing to a file.
pub trait Saveable {
    fn save_device_stats(&mut self, step: TimeS);
    fn save_data_stats(&mut self, step: TimeS);
}

/// The `Resultant` trait marks data that can be written as output. Use this to mark a struct which
/// contains the data that needs to be written to a file.
pub trait Resultant: Serialize + Copy + Clone + Debug {}

/// The `WriterOut` trait marks the output writer. Use this to mark a struct which contains the
/// writer object that writes the data to a file.
pub trait WriterOut {
    fn write_to_file<R>(&mut self, resultant: Vec<R>)
    where
        R: Resultant;
}

/// The `ResultWriter` struct contains the writer object and the file path to which the data is
/// written.
#[derive(Debug, Clone)]
pub struct GResultWriter<R, W>
where
    R: Resultant,
    W: WriterOut,
{
    pub writer: W,
    _resultant: std::marker::PhantomData<fn() -> (R, TimeS)>,
}

impl<R, W> GResultWriter<R, W>
where
    R: Resultant,
    W: WriterOut,
{
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            _resultant: std::marker::PhantomData,
        }
    }

    pub fn write_results(&mut self, resultant: Vec<R>) {
        self.writer.write_to_file(resultant);
    }
}
