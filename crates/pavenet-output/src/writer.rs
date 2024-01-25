use csv::{Writer, WriterBuilder};
use log::error;
use pavenet_engine::bucket::Resultant;
use std::fs::{File, OpenOptions};
use std::io::Seek;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum DataOutput {
    Csv(WriterCsv),
    Parquet,
}

impl DataOutput {
    pub fn new(file_name: &PathBuf) -> Self {
        match file_name.extension() {
            Some(ext) => match ext.to_str() {
                Some("csv") => Self::build_csv_writer(file_name),
                _ => panic!("Invalid file extension"),
            },
            None => panic!("Invalid file extension"),
        }
    }

    fn build_csv_writer(file_name: &PathBuf) -> DataOutput {
        // Delete file if it exists
        if file_name.exists() {
            match std::fs::remove_file(file_name) {
                Ok(_) => {}
                Err(e) => {
                    error!("Error deleting file: {}", file_name.to_str().unwrap());
                    panic!("Error deleting file: {}", e);
                }
            }
        }
        DataOutput::Csv(WriterCsv::new(file_name))
    }

    pub fn write_to_file<R: Resultant>(&mut self, data: Vec<R>) {
        match self {
            DataOutput::Csv(writer) => writer.write_to_file(data),
            DataOutput::Parquet => {}
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
            .append(false)
            .open(file_name.clone())
        {
            Ok(file) => file,
            Err(e) => {
                error!("Error opening file: {}", file_name.to_str().unwrap());
                panic!("Error opening file: {}", e);
            }
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

    fn write_to_file<R: Resultant>(&mut self, data: Vec<R>) {
        let mut writer = Self::build_writer(self.file_name.clone());
        data.iter().for_each(|record| {
            writer.serialize(record).expect("Error writing record");
        });
        writer.flush().expect("Error flushing writer");
    }
}

#[derive(Debug, Clone)]
pub struct WriterParquet {
    file_name: PathBuf,
}

impl WriterParquet {
    pub fn new(file_name: &PathBuf) -> Self {
        Self {
            file_name: file_name.to_owned(),
        }
    }

    fn write_to_file<R: Resultant>(&mut self, data: &Vec<R>) {}
}
