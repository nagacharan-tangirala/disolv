use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::str::FromStr;

use hashbrown::HashMap;
use quick_xml::{Reader, Writer};
use quick_xml::events::{BytesStart, Event};

use disolv_core::bucket::TimeMS;

use crate::produce::config::TraceSettings;
use crate::vehicles::cache::TraceInfo;
use crate::vehicles::network::NetworkReader;

pub enum TraceReader {
    Sumo(SumoReader),
    CityMoS,
}

impl TraceReader {
    pub fn new(trace_settings: &TraceSettings) -> Self {
        match trace_settings.trace_type.to_lowercase().as_str() {
            "sumo" => TraceReader::Sumo(SumoReader::new(trace_settings)),
            _ => unimplemented!("other readers not implemented"),
        }
    }

    pub fn initialize(&mut self) {
        match self {
            TraceReader::Sumo(sumo) => sumo.initialize(),
            _ => unimplemented!("only sumo trace files are supported"),
        }
    }

    pub fn read_data(&mut self, now: TimeMS) -> Option<Vec<TraceInfo>> {
        match self {
            TraceReader::Sumo(sumo) => sumo.read_positions_at(now),
            _ => unimplemented!("only sumo trace files are supported"),
        }
    }
}

pub struct SumoReader {
    reader: Reader<BufReader<File>>,
    conversion_factor: TimeMS,
    agent_id_map: HashMap<String, u64>,
    current_id: u64,
    network_reader: NetworkReader,
}

impl SumoReader {
    pub fn new(trace_settings: &TraceSettings) -> Self {
        let input_trace_file = PathBuf::from(trace_settings.input_trace.to_owned());
        let reader = Reader::from_file(input_trace_file).expect("Failed to create XML reader");
        Self {
            reader,
            conversion_factor: trace_settings.time_conversion,
            network_reader: NetworkReader::new(&trace_settings),
            agent_id_map: HashMap::new(),
            current_id: trace_settings.starting_id,
        }
    }

    fn initialize(&mut self) {
        self.network_reader.initialize();
    }

    fn read_positions_at(&mut self, now: TimeMS) -> Option<Vec<TraceInfo>> {
        let mut buffer = Vec::new();
        loop {
            match self.reader.read_event_into(&mut buffer) {
                Err(error) => panic!(
                    "Failed to read xml at position {} with error {:?}",
                    self.reader.buffer_position(),
                    error
                ),
                Ok(Event::Start(tag_begin)) => {
                    if tag_begin.name().as_ref() == b"timestep" {
                        let time_ms = self.get_time_step(&tag_begin);
                        if time_ms == now {
                            return Some(self.read_vehicle_data(now.as_u64()));
                        }
                    }
                }
                _ => {}
            }
            buffer.clear();
        }
    }

    fn get_time_step(&self, time_step_event: &BytesStart) -> TimeMS {
        // Read time attribute and convert it into ms
        let time_stamp = time_step_event
            .attributes()
            .map(|a| {
                f64::from_str(std::str::from_utf8(a.unwrap().value.as_ref()).unwrap()).unwrap()
            })
            .collect::<Vec<_>>();
        TimeMS::from((time_stamp.first().unwrap() * self.conversion_factor.as_f64()).round() as u64)
    }

    fn read_vehicle_data(&mut self, time_ms: u64) -> Vec<TraceInfo> {
        let mut temp_buffer: Vec<u8> = Vec::new();
        let mut trace_data: Vec<TraceInfo> = Vec::new();
        loop {
            let vehicle_tag_event = self
                .reader
                .read_event_into(&mut temp_buffer)
                .expect("failed to read vehicle info");

            match &vehicle_tag_event {
                Event::Empty(_) => {
                    trace_data.push(self.parse_vehicle_event(vehicle_tag_event, time_ms))
                }
                Event::End(_) => return trace_data,
                _ => {}
            }
        }
    }

    fn parse_vehicle_event(&mut self, vehicle_tag_event: Event, time_ms: u64) -> TraceInfo {
        // Convert it into a string using the writer
        let mut vehicle_buffer = Vec::new();
        let mut vehicle_writer = Writer::new(&mut vehicle_buffer);
        vehicle_writer
            .write_event(vehicle_tag_event)
            .expect("failed to write event data to buffer");
        let vehicle_str =
            String::from_utf8(vehicle_buffer).expect("failed to convert [u8] to string");

        // Parse vehicle info
        let mut trace_info = self.convert_vehicle_string(vehicle_str);
        trace_info.time_ms = time_ms;
        trace_info
    }

    fn convert_vehicle_string(&mut self, vehicle_str: String) -> TraceInfo {
        let string_pieces = vehicle_str.split_whitespace();
        let mut trace_info = TraceInfo::default();
        for part in string_pieces {
            if part.starts_with("id") {
                trace_info.agent_id = self.get_vehicle_id(part.to_string());
            }

            if part.starts_with("x") {
                let x = part.split("=").last().expect("failed to read x");
                let x_str = Self::remove_quotes(x);
                trace_info.x = f64::from_str(x_str).expect("failed to parse to float");
            }

            if part.starts_with("y") {
                let y = part.split("=").last().expect("failed to read y");
                let y_str = Self::remove_quotes(y);
                trace_info.y = f64::from_str(y_str).expect("failed to parse to float");
            }
        }
        trace_info = self.handle_offsets(trace_info);
        trace_info
    }

    fn get_vehicle_id(&mut self, id: String) -> u64 {
        let id_str = id.split("=").last().expect("failed to read id");
        let id_no_quotes = Self::remove_quotes(id_str);

        match u64::from_str(id_no_quotes) {
            Ok(val) => val,
            Err(_) => match self.agent_id_map.get(&id) {
                Some(val) => *val,
                None => {
                    self.current_id += 1;
                    self.agent_id_map.insert(id, self.current_id);
                    self.current_id
                }
            },
        }
    }

    fn remove_quotes(input: &str) -> &str {
        input.split("\"").take(2).last().expect("failed to split")
    }

    fn handle_offsets(&self, trace_info: TraceInfo) -> TraceInfo {
        let mut modified_trace_info = TraceInfo::default();
        modified_trace_info.x = self
            .network_reader
            .get_offsets()
            .subtract_x_offset(trace_info.x);
        modified_trace_info.y = self
            .network_reader
            .get_offsets()
            .subtract_y_offset(trace_info.y);
        modified_trace_info
    }
}
