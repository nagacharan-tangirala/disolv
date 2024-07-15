use arrow::datatypes::{Schema, SchemaRef};
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;
use std::fs::File;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) enum DataOutput {
    Parquet(WriterParquet),
}

impl DataOutput {
    pub fn new(file_name: &PathBuf, schema: Schema) -> Self {
        if file_name.exists() {
            match std::fs::remove_file(file_name) {
                Ok(_) => {}
                Err(e) => panic!("Error deleting file: {}", e),
            }
        }
        match file_name.extension() {
            Some(ext) => match ext.to_str() {
                Some("parquet") => DataOutput::Parquet(WriterParquet::new(file_name, schema)),
                _ => panic!("Invalid file extension"),
            },
            None => panic!("Invalid file extension"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct WriterParquet {
    pub(crate) writer: ArrowWriter<File>,
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

    pub(crate) fn close(self) {
        self.writer.close().expect("Failed to close output file");
    }
}
