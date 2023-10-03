use std::sync::mpsc::Sender;

use anyhow::Result;
use colored::Colorize;
use image::DynamicImage;
use image_hasher::Hasher;

use crate::fmt_path_for_display::fmt_path_for_display;
use crate::read_file::read_file;

#[allow(clippy::type_complexity)]
pub fn calc_img_hash(
    percent: usize,
    hasher: &Hasher,
    img_path: String,
    img_hash_result_tx: &Sender<Result<(Box<[u8]>, String), String>>,
) {
    let display_path = fmt_path_for_display(&img_path, 12);
    let img_data: Result<DynamicImage> =
        try { image::load_from_memory(&read_file(percent, &img_path)?[..])? };
    let result = match img_data {
        Ok(img) => {
            println!("{} {:>3}% {}", "[CALC]".cyan(), percent, display_path);
            let hash = Box::from(hasher.hash_image(&img).as_bytes());
            Ok((hash, img_path))
        }
        Err(e) => {
            println!(
                "{} {:>3}% Failed to open image: {} [{}]",
                "[ERR]".red(),
                percent,
                img_path,
                e
            );
            Err(img_path)
        }
    };

    img_hash_result_tx.send(result).unwrap()
}
