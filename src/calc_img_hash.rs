use std::sync::mpsc::Sender;

use colored::Colorize;
use image_hasher::HasherConfig;

use crate::read_img::read_img;

#[inline]
#[allow(clippy::type_complexity)]
pub fn calc_img_hash(
    img_path: String,
    img_hash_result_tx: &Sender<Result<(Box<[u8]>, String), String>>
) {
    let hasher = HasherConfig::new().to_hasher();

    let result = match read_img(&img_path) {
        Ok(img) => {
            println!("{} {}", "[CALC]".cyan(), img_path);
            let hash = Box::from(
                hasher
                    .hash_image(&img)
                    .as_bytes()
            );
            Ok((hash, img_path))
        }
        Err(e) => {
            println!(
                "{} Failed to open image: {} [{}]",
                "[ERR]".red(),
                img_path,
                e
            );
            Err(img_path)
        }
    };

    img_hash_result_tx
        .send(result)
        .unwrap()
}
