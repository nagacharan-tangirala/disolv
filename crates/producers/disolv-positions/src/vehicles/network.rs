use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;

use log::debug;
use quick_xml::{Reader, Writer};
use quick_xml::events::Event;
use typed_builder::TypedBuilder;

use crate::produce::config::TraceSettings;

pub enum NetworkReader {
    Sumo(SumoNet),
}

impl NetworkReader {
    pub fn new(trace_settings: &TraceSettings) -> Self {
        match trace_settings.trace_type.as_str() {
            "sumo" => NetworkReader::Sumo(SumoNet::new(trace_settings)),
            _ => panic!("Other network files are not supported"),
        }
    }

    pub fn initialize(&mut self) {
        match self {
            NetworkReader::Sumo(sumo_net) => sumo_net.initialize(),
        }
    }

    pub fn get_offsets(&self) -> SumoOffsets {
        match self {
            NetworkReader::Sumo(sumo_net) => sumo_net.offsets,
        }
    }

    pub fn projection_string(&self) -> &str {
        match self {
            NetworkReader::Sumo(sumo_net) => &sumo_net.projection_string,
        }
    }
}

#[derive(Copy, Clone, Default, TypedBuilder)]
pub struct SumoOffsets {
    x_offset: f64,
    y_offset: f64,
}

impl SumoOffsets {
    pub fn subtract_x_offset(&self, input: f64) -> f64 {
        input - self.x_offset
    }

    pub fn subtract_y_offset(&self, input: f64) -> f64 {
        input - self.y_offset
    }
}

pub struct SumoNet {
    net_file: Reader<BufReader<File>>,
    offsets: SumoOffsets,
    projection_string: String,
}

impl SumoNet {
    pub fn new(trace_settings: &TraceSettings) -> Self {
        let net_file =
            Reader::from_file(&trace_settings.input_network).expect("Failed to create XML reader");
        Self {
            net_file,
            offsets: SumoOffsets::default(),
            projection_string: String::new(),
        }
    }

    pub fn initialize(&mut self) {
        let mut buffer = Vec::new();
        loop {
            let event = self.net_file.read_event_into(&mut buffer);
            match event.clone() {
                Err(error) => panic!(
                    "Failed to read xml at position {} with error {:?}",
                    self.net_file.buffer_position(),
                    error
                ),
                Ok(Event::Empty(tag_begin)) => {
                    if tag_begin.name().as_ref() == b"location" {
                        self.read_network_data(&event.expect("failed to extract event"));
                        return;
                    }
                    debug!("tag_begin {:?}", tag_begin.name());
                }
                _ => {}
            }
            buffer.clear();
        }
    }

    pub fn read_network_data(&mut self, location_event: &Event) {
        match &location_event {
            Event::Empty(_) => {
                let mut location_buffer = Vec::new();
                let mut loc_writer = Writer::new(&mut location_buffer);
                loc_writer
                    .write_event(location_event.to_owned())
                    .expect("failed to write event data to buffer");
                let location_tag =
                    String::from_utf8(location_buffer).expect("failed to convert [u8] to string");
                self.read_location_tag(location_tag);
            }
            Event::End(_) => (),
            _ => {}
        }
    }

    pub fn read_location_tag(&mut self, location: String) {
        debug!("Location string {}", location);
        let pieces = location.split("convBoundary");
        for part in pieces {
            if part.contains("netOffset") {
                self.read_offsets(part.to_string());
            }

            if part.contains("proj") {
                self.read_projection(part.to_string());
            }
        }
    }

    pub fn read_offsets(&mut self, offsets: String) {
        debug!("offset string {}", offsets);
        let offsets = offsets.split("=").last().expect("failed to read offset");
        let offsets = offsets.replace("\"", "");

        let x_offset = offsets
            .split(",")
            .take(1)
            .last()
            .expect("failed to read x offset");
        let y_offset = offsets.split(",").last().expect("failed to read y offset");

        let x = f64::from_str(x_offset).expect("failed to parse x");
        let y = f64::from_str(y_offset.trim()).expect("failed to parse y");

        self.offsets = SumoOffsets::builder().x_offset(x).y_offset(y).build();
    }

    pub fn read_projection(&mut self, location_tag: String) {
        let proj = location_tag
            .split("projParameter=")
            .last()
            .expect("failed to read proj");
        self.projection_string = proj.replace("/>", "");
        debug!("{}", self.projection_string);
    }
}
