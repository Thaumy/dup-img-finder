use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use colored::Colorize;
use image::DynamicImage;
use image_hasher::{HashBytes, Hasher};

use crate::cache::Cache;
use crate::fmt_path_for_display::fmt_path_for_display;
use crate::read_file::read_file;

#[allow(clippy::type_complexity)]
pub fn calc_img_hash(
    cache: &Arc<Mutex<Cache>>,
    percent: usize,
    hasher: &Hasher,
    img_path: String,
    img_hash_result_tx: &Sender<Result<(Box<[u8]>, String), String>>,
) {
    let display_path = fmt_path_for_display(&img_path, 12);

    // query cache
    {
        let result = {
            let cache = cache.lock().unwrap();
            cache.query(&img_path)
        };
        match result {
            Ok(Some(hash)) => {
                println!("{} {:>3}% {}", "[HIT]".green(), percent, display_path);
                let result = Ok((hash, img_path));
                return img_hash_result_tx.send(result).unwrap();
            }
            Err(e) => {
                println!(
                    "{} {:>3}% Failed to query cache in database: {} [{}]",
                    "[ERR]".red(),
                    percent,
                    img_path,
                    e
                );
                return img_hash_result_tx.send(Err(img_path)).unwrap();
            }
            _ => (),
        };
    }

    let img_data: Result<DynamicImage> =
        try { image::load_from_memory(&read_file(percent, &img_path)?[..])? };

    let result = match img_data {
        Ok(img) => {
            println!("{} {:>3}% {}", "[CALC]".cyan(), percent, display_path);
            let hash: Box<[u8]> = Box::from(hasher.hash_image(&img).as_bytes());
            // insert cache
            {
                let cache = cache.lock().unwrap();
                if let Err(e) = cache.insert(&img_path, hash.as_slice()) {
                    println!(
                        "{} {:>3}% Failed to insert hash to database: {} [{}]",
                        "[ERR]".red(),
                        percent,
                        img_path,
                        e
                    );
                }
            }
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
