use csv::{Writer, WriterBuilder};
use log::debug;
use pavenet_engine::result::{Resultant, WriterOut};
use std::fs::{File, OpenOptions};
use std::io::Seek;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum FileOutput {
    Csv(WriterCsv),
    Parquet,
}

impl FileOutput {
    pub fn new(file_name: &PathBuf) -> Self {
        match file_name.extension() {
            Some(ext) => match ext.to_str() {
                Some("csv") => Self::build_csv_writer(file_name),
                _ => panic!("Invalid file extension"),
            },
            None => panic!("Invalid file extension"),
        }
    }

    fn build_csv_writer(file_name: &PathBuf) -> FileOutput {
        FileOutput::Csv(WriterCsv::new(file_name))
    }
}

impl WriterOut for FileOutput {
    fn write_to_file<R>(&mut self, resultant: Vec<R>)
    where
        R: Resultant,
    {
        match self {
            FileOutput::Csv(writer) => writer.write_to_file(resultant),
            FileOutput::Parquet => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WriterCsv {
    file_name: PathBuf,
}

impl WriterCsv {
    pub fn new(file_name: &PathBuf) -> Self {
        Self {
            file_name: file_name.to_owned(),
        }
    }

    fn build_writer(file_name: PathBuf) -> Writer<File> {
        let mut file = match OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(file_name.to_owned())
        {
            Ok(file) => file,
            Err(e) => panic!("Error opening file: {}", e),
        };

        let needs_headers: bool = match file.seek(std::io::SeekFrom::End(0)) {
            Ok(pos) => pos == 0,
            Err(e) => panic!("Error seeking file: {}", e),
        };

        let writer = WriterBuilder::new()
            .has_headers(needs_headers)
            .from_writer(file);
        writer
    }

    pub fn write_to_file<R>(&mut self, resultant: Vec<R>)
    where
        R: Resultant,
    {
        let mut writer = Self::build_writer(self.file_name.to_owned());
        debug!("Writing to file: {:?}", self.file_name);
        debug!("Writing {} records", resultant.len());
        for result in resultant {
            writer.serialize(result).unwrap();
        }
    }
}
