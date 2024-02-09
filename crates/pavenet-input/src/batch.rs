use crate::columns::TIME_STEP;
use arrow_array::RecordBatch;
use log::{debug, error};
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::file::statistics::Statistics;
use pavenet_engine::bucket::TimeMS;
use std::fs::File;
use std::path::PathBuf;

pub(crate) fn read_f64_column(col_name: &str, record_batch: &RecordBatch) -> Vec<f64> {
    let array_data = match record_batch.column_by_name(col_name) {
        Some(column) => column.to_data(),
        None => panic!("Failed to read column {}", col_name),
    };
    let mut column_data = vec![];
    for i in 0..array_data.buffers().len() {
        column_data.extend(array_data.buffer::<f64>(i).to_vec());
    }
    column_data
}

pub(crate) fn read_u32_column(col_name: &str, record_batch: &RecordBatch) -> Vec<u32> {
    let array_data = match record_batch.column_by_name(col_name) {
        Some(column) => column.to_data(),
        None => panic!("Failed to read column {}", col_name),
    };
    let mut column_data = vec![];
    for i in 0..array_data.buffers().len() {
        column_data.extend(array_data.buffer::<u32>(i).to_vec());
    }
    column_data
}

pub(crate) fn read_u64_column(col_name: &str, record_batch: &RecordBatch) -> Vec<u64> {
    let array_data = match record_batch.column_by_name(col_name) {
        Some(column) => column.to_data(),
        None => panic!("Failed to read column {}", col_name),
    };
    let mut column_data = vec![];
    for i in 0..array_data.buffers().len() {
        column_data.extend(array_data.buffer::<u64>(i).to_vec());
    }
    column_data
}

pub(crate) fn get_row_groups_for_time(
    file_path: &PathBuf,
    is_streaming: bool,
    start_time: TimeMS,
    end_time: TimeMS,
) -> Vec<usize> {
    let par_file = File::open(file_path).unwrap();
    let reader = SerializedFileReader::new(par_file).unwrap();
    if !is_streaming {
        return (0..reader.num_row_groups()).collect();
    }
    let mut interested_groups: Vec<usize> = vec![];

    'row_group_loop: for row_group in 0..reader.num_row_groups() {
        let row_group_reader = match reader.get_row_group(row_group) {
            Ok(row_group_reader) => row_group_reader,
            Err(e) => {
                debug!("Error reading file: {}", e);
                panic!("Error reading file: {}", e);
            }
        };

        for column in row_group_reader.metadata().columns() {
            if column.column_descr().name() == TIME_STEP {
                match column.statistics() {
                    Some(Statistics::Int64(stats)) => {
                        if start_time.as_i64() > *stats.max() {
                            continue 'row_group_loop;
                        }
                        if end_time.as_i64() < *stats.min() {
                            break 'row_group_loop;
                        }
                        interested_groups.push(row_group);
                    }
                    None => {
                        break 'row_group_loop;
                    }
                    _ => {
                        error!(
                            "Time step column is not of type int64 in file {}",
                            file_path.to_str().unwrap()
                        );
                        panic!(
                            "Time step column is not of type int64 in file {}",
                            file_path.to_str().unwrap()
                        )
                    }
                }
                break;
            }
        }
    }
    interested_groups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_row_groups_for_time() {
        let parquet_file = "/mnt/hdd/workspace/pavenet/input/medium/links/r2v_links.parquet";
        let parquet_filepath = PathBuf::from(parquet_file);
        let start_time = TimeMS::from(3750000);
        let end_time = TimeMS::from(3799900);
        let row_groups = get_row_groups_for_time(&parquet_filepath, true, start_time, end_time);
        println!("Row groups: {:?}", row_groups);
        assert_eq!(row_groups.len(), 1);
    }
}
