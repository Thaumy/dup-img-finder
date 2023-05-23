use std::sync::mpsc::Sender;

use anyhow::Result;
use colored::Colorize;
use image_hasher::HasherConfig;

#[inline]
pub fn calc_img_hash(
    img_path: String,
    img_hash_result_tx: &Sender<
        Result<(Box<[u8]>, String), String>
    >
) {
    let hasher = HasherConfig::new().to_hasher();

    let result = match image::open(img_path.as_str()) {
        Ok(img) => {
            let hash = Box::from(
                hasher
                    .hash_image(&img)
                    .as_bytes()
            );
            println!("{} {}", "[CALC]".cyan(), img_path);
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
