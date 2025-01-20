use std::path::PathBuf;

use burn::data::dataloader::batcher::Batcher;
use burn::data::dataset::vision::PixelDepth;
use burn::data::dataset::{Dataset, HuggingfaceDatasetLoader};
use burn::prelude::Backend;
use burn::tensor::{Device, ElementConversion, Int, Shape, Tensor, TensorData};
use image::{load_from_memory, ColorType, DynamicImage};
use rusqlite::Connection;

use crate::models::ai::common::BatchType;

const WIDTH: usize = 32;
const HEIGHT: usize = 32;

#[derive(Clone, Debug)]
pub struct CifarItem {
    image: Vec<u8>,
    label: u8,
}

impl CifarItem {
    pub(crate) fn label(&self) -> u8 {
        self.label
    }
}

#[derive(Clone, Default, Debug)]
pub struct CifarFlDataset {
    pub images: Vec<CifarItem>,
}

impl CifarFlDataset {
    pub fn new(batch_type: BatchType) -> Self {
        let images = match batch_type {
            BatchType::Train => Self::build_train_dataset(),
            BatchType::Test => Self::build_test_dataset(),
        };
        Self { images }
    }

    pub fn with_images(images: Vec<CifarItem>) -> Self {
        Self { images }
    }

    fn build_test_dataset() -> Vec<CifarItem> {
        let dataset = HuggingfaceDatasetLoader::new("uoft-cs/cifar10");
        let sql_file = dataset.db_file().expect("failed to get db file");
        Self::read_from_db(&sql_file, "test")
    }

    fn build_train_dataset() -> Vec<CifarItem> {
        let dataset = HuggingfaceDatasetLoader::new("uoft-cs/cifar10");
        let sql_file = dataset.db_file().expect("failed to get db file");
        Self::read_from_db(&sql_file, "train")
    }

    fn read_from_db(sql_file: &PathBuf, split: &str) -> Vec<CifarItem> {
        let sql_connection = Connection::open(sql_file).expect("failed to connect");
        let table_size = Self::get_table_size(sql_file, split);
        let mut cifar_items = Vec::new();

        let mut select_str = "SELECT img_bytes, label FROM ".to_string();
        select_str.push_str(split);
        select_str.push_str(" WHERE row_id = ?1");

        for i in 1..=table_size {
            let mut select_statement = sql_connection
                .prepare(select_str.as_str())
                .expect("failed to prepare stmt");

            let image_as_vec: Vec<u8> = select_statement
                .query_row([i], |row| row.get(0))
                .expect("failed to read");
            let label: u8 = select_statement
                .query_row([i], |row| row.get(1))
                .expect("failed to read");

            let dynamic_image = load_from_memory(&image_as_vec).expect("failed to read");
            let pixel_vec = Self::convert(dynamic_image);
            let image_item = Self::image_as_vec_u8(pixel_vec);
            let cifar_item = CifarItem {
                image: image_item,
                label,
            };
            cifar_items.push(cifar_item);
        }
        cifar_items
    }

    fn get_table_size(sql_file: &PathBuf, split: &str) -> usize {
        let sql_connection = Connection::open(sql_file).expect("failed to connect");
        let mut statement = "SELECT MAX(row_id) from ".to_string();
        statement.push_str(split);
        let mut count_statement = sql_connection
            .prepare(&statement)
            .expect("failed to create count statement");
        let item: usize = count_statement
            .query_row([], |row| row.get(0))
            .expect("failed to read");
        item
    }

    fn image_as_vec_u8(image: Vec<PixelDepth>) -> Vec<u8> {
        image
            .into_iter()
            .map(|p: PixelDepth| -> u8 { p.try_into().unwrap() })
            .collect::<Vec<u8>>()
    }

    fn convert(image: DynamicImage) -> Vec<PixelDepth> {
        let img_vec = match image.color() {
            ColorType::L8 => image
                .into_luma8()
                .iter()
                .map(|&x| PixelDepth::U8(x))
                .collect(),
            ColorType::La8 => image
                .into_luma_alpha8()
                .iter()
                .map(|&x| PixelDepth::U8(x))
                .collect(),
            ColorType::L16 => image
                .into_luma16()
                .iter()
                .map(|&x| PixelDepth::U16(x))
                .collect(),
            ColorType::La16 => image
                .into_luma_alpha16()
                .iter()
                .map(|&x| PixelDepth::U16(x))
                .collect(),
            ColorType::Rgb8 => image
                .into_rgb8()
                .iter()
                .map(|&x| PixelDepth::U8(x))
                .collect(),
            ColorType::Rgba8 => image
                .into_rgba8()
                .iter()
                .map(|&x| PixelDepth::U8(x))
                .collect(),
            ColorType::Rgb16 => image
                .into_rgb16()
                .iter()
                .map(|&x| PixelDepth::U16(x))
                .collect(),
            ColorType::Rgba16 => image
                .into_rgba16()
                .iter()
                .map(|&x| PixelDepth::U16(x))
                .collect(),
            ColorType::Rgb32F => image
                .into_rgb32f()
                .iter()
                .map(|&x| PixelDepth::F32(x))
                .collect(),
            ColorType::Rgba32F => image
                .into_rgba32f()
                .iter()
                .map(|&x| PixelDepth::F32(x))
                .collect(),
            _ => panic!("Unrecognized image color type"),
        };
        img_vec
    }

    pub fn split_with_ratios(ratios: Vec<f64>, batch_type: BatchType) -> Vec<CifarFlDataset> {
        let all_data = CifarFlDataset::new(batch_type).to_owned();
        let total_samples = all_data.len();
        let mut data_chunks = Vec::new();
        let chunk_sizes: Vec<usize> = ratios
            .iter()
            .map(|r| (r * total_samples as f64).round() as usize)
            .collect();

        let mut start = 0;
        for size in chunk_sizes {
            if start >= total_samples {
                break;
            }
            let end = usize::min(start + size, total_samples);
            let chunk = all_data.images[start..end].to_vec();
            data_chunks.push(CifarFlDataset::with_images(chunk));
            start = end;
        }
        data_chunks
    }
}

impl Dataset<CifarItem> for CifarFlDataset {
    fn get(&self, index: usize) -> Option<CifarItem> {
        self.images.get(index).cloned()
    }

    fn len(&self) -> usize {
        self.images.len()
    }
}

// CIFAR-10 mean and std values
const MEAN: [f32; 3] = [0.4914, 0.48216, 0.44653];
const STD: [f32; 3] = [0.24703, 0.24349, 0.26159];

/// Normalizer for the CIFAR-10 dataset.
#[derive(Clone)]
pub struct Normalizer<B: Backend> {
    pub mean: Tensor<B, 4>,
    pub std: Tensor<B, 4>,
}

impl<B: Backend> Normalizer<B> {
    /// Creates a new normalizer.
    pub fn new(device: &Device<B>) -> Self {
        let mean = Tensor::<B, 1>::from_floats(MEAN, device).reshape([1, 3, 1, 1]);
        let std = Tensor::<B, 1>::from_floats(STD, device).reshape([1, 3, 1, 1]);
        Self { mean, std }
    }

    /// Normalizes the input image according to the CIFAR-10 dataset.
    ///
    /// The input image should be in the range [0, 1].
    /// The output image will be in the range [-1, 1].
    ///
    /// The normalization is done according to the following formula:
    /// `input = (input - mean) / std`
    pub fn normalize(&self, input: Tensor<B, 4>) -> Tensor<B, 4> {
        (input - self.mean.clone()) / self.std.clone()
    }
}
#[derive(Debug, Clone)]
pub struct CifarBatch<B: Backend> {
    pub images: Tensor<B, 4>,
    pub targets: Tensor<B, 1, Int>,
}

#[derive(Clone)]
pub struct CifarBatcher<B: Backend> {
    normalizer: Normalizer<B>,
    device: B::Device,
}

impl<B: Backend> CifarBatcher<B> {
    pub fn new(device: B::Device, normalizer: Normalizer<B>) -> Self {
        Self { device, normalizer }
    }
}

impl<B: Backend> Batcher<CifarItem, CifarBatch<B>> for CifarBatcher<B> {
    fn batch(&self, items: Vec<CifarItem>) -> CifarBatch<B> {
        let targets = items
            .iter()
            .map(|item| {
                Tensor::<B, 1, Int>::from_data(
                    TensorData::from([(item.label as i64).elem::<B::IntElem>()]),
                    &self.device,
                )
            })
            .collect();

        let images = items
            .into_iter()
            .map(|item| TensorData::new(item.image, Shape::new([32, 32, 3])))
            .map(|data| {
                Tensor::<B, 3>::from_data(data.convert::<B::FloatElem>(), &self.device)
                    // permute(2, 0, 1)
                    .swap_dims(2, 1) // [H, C, W]
                    .swap_dims(1, 0) // [C, H, W]
            })
            .map(|tensor| tensor / 255) // normalize between [0, 1]
            .collect();

        let images = Tensor::stack(images, 0);
        let targets = Tensor::cat(targets, 0);

        let images = self.normalizer.normalize(images);

        CifarBatch { images, targets }
    }
}
