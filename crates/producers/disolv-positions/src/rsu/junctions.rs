use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::str::FromStr;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use disolv_core::bucket::TimeMS;
use disolv_output::result::ResultWriter;

use crate::produce::config::{RSUSettings, TimingSettings};
use crate::rsu::activation::RSUActivations;
use crate::rsu::writer::{RSUInfo, RSUWriter};
use crate::vehicles::offset::OffsetReader;

pub(crate) enum RSUPlacement {
    SumoJunctions(RSUJunctions),
}

impl RSUPlacement {
    pub(crate) fn new(rsu_settings: &RSUSettings, timing_settings: &TimingSettings) -> Self {
        match rsu_settings.placement_type.to_lowercase().as_str() {
            "sumojunction" => {
                RSUPlacement::SumoJunctions(RSUJunctions::new(&rsu_settings, timing_settings))
            }
            _ => panic!("only junctions type of placement is supported"),
        }
    }

    pub(crate) fn initialize(&mut self) {
        match self {
            RSUPlacement::SumoJunctions(junctions) => junctions.initialize(),
        }
    }

    pub(crate) fn read_data(&mut self, _time_ms: TimeMS) {
        match self {
            RSUPlacement::SumoJunctions(_) => {}
        }
    }

    pub(crate) fn complete(self) {
        match self {
            RSUPlacement::SumoJunctions(sumo) => sumo.complete(),
        }
    }
}

pub(crate) struct RSUJunctions {
    net_reader: Reader<BufReader<File>>,
    starting_id: u64,
    rsu_writer: RSUWriter,
    rsu_activations: RSUActivations,
    offset_helper: OffsetReader,
    duration: u64,
}

impl RSUJunctions {
    fn new(rsu_settings: &RSUSettings, timing_settings: &TimingSettings) -> Self {
        let net_file = PathBuf::from(rsu_settings.input_network.to_owned());
        let net_reader = Reader::from_file(net_file).expect("Failed to create XML reader");
        Self {
            net_reader,
            starting_id: rsu_settings.starting_id,
            rsu_writer: RSUWriter::new(rsu_settings),
            offset_helper: OffsetReader::new(&rsu_settings.input_network),
            rsu_activations: RSUActivations::new(rsu_settings),
            duration: timing_settings.duration.as_u64(),
        }
    }

    fn initialize(&mut self) {
        self.offset_helper.initialize();
        self.read_junctions();
    }

    fn read_junctions(&mut self) {
        let mut buffer = Vec::new();
        loop {
            let event = self.net_reader.read_event_into(&mut buffer);
            match event.clone() {
                Err(error) => panic!(
                    "Failed to read xml at position {} with error {:?}",
                    self.net_reader.buffer_position(),
                    error
                ),
                Ok(Event::Start(valid_tag_begin)) => {
                    if valid_tag_begin.name().as_ref() == b"junction" {
                        self.read_junction_data(&event.expect("failed to read event"));
                    }
                }
                Ok(Event::Empty(_)) => {
                    // non-priority junctions are read here, we can ignore them.
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buffer.clear();
        }
    }

    fn read_junction_data(&mut self, junction_event: &Event) {
        match &junction_event {
            Event::Start(event_begin) => {
                if self.is_valid_junction(event_begin) {
                    let (x, y) = self.read_coords_from_file(event_begin);
                    self.offset_and_store(x, y);
                }
            }
            Event::End(_) => (),
            _ => {}
        }
    }

    fn read_coords_from_file(&mut self, junction_tag: &BytesStart) -> (f64, f64) {
        let mut x = 0.0;
        let mut y = 0.0;
        junction_tag.attributes().for_each(|attr| match attr {
            Ok(data) => {
                let key =
                    String::from_utf8(data.key.as_ref().to_vec()).expect("failed to read key");
                if key == "x" {
                    x = f64::from_str(
                        String::from_utf8(data.value.to_vec())
                            .expect("failed to read value")
                            .as_str(),
                    )
                    .expect("failed to parse x coordinate");
                }
                if key == "y" {
                    y = f64::from_str(
                        String::from_utf8(data.value.to_vec())
                            .expect("failed to read value")
                            .as_str(),
                    )
                    .expect("failed to parse y coordinate");
                }
            }
            Err(_) => panic!("failed to read attributes of a junction {:?}", junction_tag),
        });
        (x, y)
    }

    fn offset_and_store(&mut self, x: f64, y: f64) {
        let rsu_id = self.get_rsu_id();
        self.rsu_activations.store_activation(rsu_id, self.duration);

        let x = self.offset_helper.peek_offsets().subtract_x_offset(x);
        let y = self.offset_helper.peek_offsets().subtract_y_offset(y);

        let rsu_info = RSUInfo::builder()
            .time_ms(0)
            .agent_id(rsu_id)
            .x(x)
            .y(y)
            .build();
        self.rsu_writer.store_info(rsu_info);
    }

    fn get_rsu_id(&mut self) -> u64 {
        self.starting_id += 1;
        self.starting_id
    }

    fn is_valid_junction(&self, junction_tag: &BytesStart) -> bool {
        let mut is_valid = false;
        junction_tag.attributes().for_each(|attr| match attr {
            Ok(data) => {
                let value = String::from_utf8(data.value.to_vec())
                    .expect("failed to read junction type attribute");
                if value.contains("priority") {
                    is_valid = true;
                }
            }
            Err(_) => panic!("failed to read attributes of a junction {:?}", junction_tag),
        });
        is_valid
    }

    fn complete(self) {
        self.rsu_activations.close_file();
        self.rsu_writer.close_file();
    }
}
