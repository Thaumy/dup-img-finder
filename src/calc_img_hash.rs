use std::sync::mpsc::Sender;

use colored::Colorize;
use image_hasher::HasherConfig;

use crate::fmt_path_for_display::fmt_path_for_display;
use crate::read_img::read_img;

#[inline]
#[allow(clippy::type_complexity)]
pub fn calc_img_hash(
    percent: usize,
    img_path: String,
    img_hash_result_tx: &Sender<Result<(Box<[u8]>, String), String>>
) {
    let hasher = HasherConfig::new().to_hasher();

    let display_path = fmt_path_for_display(&img_path, 12);
    let result = match read_img(percent, &img_path) {
        Ok(img) => {
            println!(
                "{} {:>3}% {}",
                "[CALC]".cyan(),
                percent,
                display_path
            );
            let hash = Box::from(
                hasher
                    .hash_image(&img)
                    .as_bytes()
            );
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

    img_hash_result_tx
        .send(result)
        .unwrap()
}
