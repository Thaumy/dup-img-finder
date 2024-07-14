#![feature(try_blocks)]
#![warn(clippy::all, clippy::nursery, clippy::cargo_common_metadata)]

mod cache;
mod calc_img_hash;
mod fmt_path_for_display;
mod get_img_paths;
mod infra;
mod read_file;
mod settings;
mod symlink_dup_files;
mod symlink_err_files;

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::{thread, vec};

use anyhow::Result;
use cache::Cache;
use clap::Parser;
use image_hasher::HasherConfig;

use crate::calc_img_hash::calc_img_hash;
use crate::get_img_paths::get_img_paths;
use crate::infra::WrapResult;
use crate::settings::args::Args;
use crate::settings::cfg::Config;
use crate::symlink_dup_files::symlink_dup_files;
use crate::symlink_err_files::symlink_err_files;

fn main() -> Result<()> {
    let args: Args = Args::parse();

    let thread_count = args.threads.unwrap_or_else(num_cpus::get);

    let config = Config::read()?;
    let cache = Arc::new(Mutex::new(Cache::new(&config.cache)?));

    let img_paths = &get_img_paths(config, args.input_path)?;
    let (img_hash_result_tx, img_hash_result_rx) = channel();
    let total_img_count = img_paths.len() as f64;
    let calc_img_count = &AtomicUsize::new(0);

    let hasher = &HasherConfig::new().to_hasher();

    let (dup_img_hash_paths, err_img_paths) = thread::scope(|s| {
        for _ in 0..thread_count {
            s.spawn({
                let img_hash_result_tx = img_hash_result_tx.clone();
                let cache = cache.clone();

                move || {
                    while let Some(img_path) = img_paths.pop() {
                        let calc_img_count = calc_img_count.fetch_add(1, Ordering::SeqCst) as f64;
                        let percent = (calc_img_count / total_img_count * 100.0).round() as usize;
                        calc_img_hash(&cache, percent, hasher, img_path, &img_hash_result_tx);
                    }
                }
            });
        }
        drop(img_hash_result_tx);

        let mut err_img_paths = vec![];
        let mut img_hash_map = BTreeMap::new();

        for result in img_hash_result_rx {
            match result {
                Ok((hash, path)) => (*img_hash_map.entry(hash).or_insert(vec![])).push(path),
                Err(msg) => err_img_paths.push(msg),
            }
        }
        img_hash_map.retain(|_, vec| vec.len() > 1);

        (img_hash_map, err_img_paths)
    });

    println!();

    symlink_err_files(&args.output_path, err_img_paths.as_slice())?;
    symlink_dup_files(
        &args.output_path,
        dup_img_hash_paths
            .iter()
            .map(|x| (x.0.as_ref(), x.1.as_slice())),
    )?;

    ().wrap_ok()
}
