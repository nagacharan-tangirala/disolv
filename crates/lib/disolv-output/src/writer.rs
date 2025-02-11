use std::fs::File;
use std::path::PathBuf;

use arrow::csv::Writer;
use arrow::datatypes::{Schema, SchemaRef};
use arrow::record_batch::{RecordBatch, RecordBatchWriter};
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;

#[derive(Debug)]
pub enum WriterType {
    Parquet(WriterParquet),
    Csv(WriterCsv),
}

impl WriterType {
    pub fn new(file_name: &PathBuf, schema: Schema) -> Self {
        if file_name.exists() {
            match std::fs::remove_file(file_name) {
                Ok(_) => {}
                Err(e) => panic!("Error deleting file: {}", e),
            }
        }
        match file_name.extension() {
            Some(ext) => match ext.to_str() {
                Some("parquet") => WriterType::Parquet(WriterParquet::new(file_name, schema)),
                Some("csv") => WriterType::Csv(WriterCsv::new(file_name)),
                _ => panic!("Invalid file extension"),
            },
            None => panic!("Invalid file extension"),
        }
    }

    pub fn record_batch_to_file(&mut self, record_batch: &RecordBatch) {
        match self {
            WriterType::Parquet(par_writer) => par_writer
                .writer
                .write(record_batch)
                .expect("Failed to write record batches to parquet"),
            WriterType::Csv(csv_writer) => csv_writer
                .writer
                .write(record_batch)
                .expect("Failed to write record batches to csv"),
        }
    }

    pub fn close(self) {
        match self {
            WriterType::Parquet(writer) => writer.close(),
            WriterType::Csv(writer) => writer.close(),
        }
    }
}

#[derive(Debug)]
pub struct WriterParquet {
    pub writer: ArrowWriter<File>,
}

impl WriterParquet {
    fn new(file_name: &PathBuf, schema: Schema) -> Self {
        let props = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .build();
        let output_file = match File::create(file_name) {
            Ok(file) => file,
            Err(_) => panic!("Failed to create links file to write"),
        };
        let writer = match ArrowWriter::try_new(output_file, SchemaRef::from(schema), Some(props)) {
            Ok(writer) => writer,
            Err(_) => panic!("Failed to create links file writer"),
        };
        Self { writer }
    }

    pub fn close(self) {
        self.writer.close().expect("Failed to close parquet file");
    }
}

#[derive(Debug)]
pub struct WriterCsv {
    pub writer: Writer<File>,
}

impl WriterCsv {
    fn new(file_name: &PathBuf) -> Self {
        let writer = Writer::new(File::create(file_name).expect("failed to create file"));
        Self { writer }
    }

    pub fn close(self) {
        self.writer.close().expect("failed to close csv file");
    }
}
