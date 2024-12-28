use std::fmt::{Display, Formatter, write};
use std::path::PathBuf;

use typed_builder::TypedBuilder;

use crate::simulation::config::PositionFiles;

#[derive(Clone)]
pub enum TraceReader {
    Sumo(SumoReader),
    CityMoS,
}

impl TraceReader {
    pub fn new(position_files: &PositionFiles) -> Self {
        match position_files.trace_type.to_lowercase().as_str() {
            "sumo" => TraceReader::Sumo(SumoReader::new(position_files)),
            _ => unimplemented!("other readers not implemented"),
        }
    }

    pub fn initialize(&mut self) {
        match self {
            TraceReader::Sumo(sumo) => sumo.initialize(),
            _ => unimplemented!("only sumo trace files are supported"),
        }
    }
}

#[derive(Clone, TypedBuilder)]
pub struct SumoReader {
    pub trace_file: PathBuf,
}

impl SumoReader {
    pub fn new(trace_settings: &PositionFiles) -> Self {
        Self {
            trace_file: PathBuf::from(trace_settings.trace.to_owned()),
        }
    }

    pub fn initialize(&mut self) {}
}
