#![feature(try_blocks)]
#![warn(clippy::all, clippy::nursery, clippy::cargo_common_metadata)]

mod calc_img_hash;
mod fmt_path_for_display;
mod get_img_paths;
mod infra;
mod read_file;
mod settings;
mod symlink_dup_files;
mod symlink_err_files;

use std::collections::BTreeMap;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Mutex;
use std::{thread, vec};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use crossbeam::queue::SegQueue;
use image_hasher::HasherConfig;

use crate::calc_img_hash::calc_img_hash;
use crate::fmt_path_for_display::fmt_path_for_display;
use crate::get_img_paths::get_img_paths;
use crate::infra::result::WrapResult;
use crate::read_file::read_file;
use crate::settings::args::Args;
use crate::settings::cfg::Config;
use crate::symlink_dup_files::symlink_dup_files;
use crate::symlink_err_files::symlink_err_files;

const TASK_PER_THREAD: usize = 20;

fn main() -> Result<()> {
    let args: Args = Args::parse();

    let img_paths = get_img_paths(Config::read()?, args.input_path)?;
    let img_data_paths = SegQueue::<(Result<Vec<u8>>, String)>::new();
    let has_more_data = &AtomicBool::new(true);

    let img_hashes = SegQueue::new();
    let total_img_count = img_paths.len();
    let calc_img_count = &AtomicUsize::new(0);

    let hasher = &HasherConfig::new().to_hasher();

    let calc_percent = || {
        let calc_img_count = calc_img_count.fetch_add(1, Relaxed) as f64;
        // Mul 2 for both read and calc increasing
        let total_img_count = total_img_count as f64 * 2.0;
        calc_img_count / total_img_count * 100.0
    };

    let err_img_paths = Mutex::new(vec![]);
    let read_thread = &thread::current();

    thread::scope(|s| {
        let thread_count = args.threads.unwrap_or_else(num_cpus::get);

        for _ in 0..thread_count {
            let img_data_paths = &img_data_paths;
            let img_hashes = &img_hashes;
            let err_img_paths = &err_img_paths;
            s.spawn(move || {
                while has_more_data.load(Acquire) {
                    while let Some((data, path)) = img_data_paths.pop() {
                        match calc_img_hash(hasher, data) {
                            Ok(hash) => {
                                let display_path = fmt_path_for_display(&path, 13);
                                println!(
                                    "{} {:0>4.1}% {}",
                                    "[CALC]".cyan(),
                                    calc_percent(),
                                    display_path
                                );
                                img_hashes.push((hash, path));
                            }
                            Err(e) => {
                                println!(
                                    "{} {:0>4.1}% Failed to open image: {} [{}]",
                                    "[ERR]".red(),
                                    calc_percent(),
                                    path,
                                    e
                                );
                                err_img_paths.lock().unwrap().push(path);
                            }
                        };
                        if img_data_paths.len() < thread_count * TASK_PER_THREAD {
                            read_thread.unpark();
                        }
                    }
                    std::hint::spin_loop();
                }
            });
        }

        tokio_uring::start(async {
            for path in img_paths {
                let display_path = fmt_path_for_display(&path, 13);
                println!(
                    "{} {:0>4.1}% {}",
                    "[READ]".blue(),
                    calc_percent(),
                    display_path
                );

                let data = read_file(path.clone()).await;
                img_data_paths.push((data, path));
                if img_data_paths.len() > thread_count * TASK_PER_THREAD {
                    thread::park()
                }
            }
            has_more_data.store(false, Release);
        });
    });

    let dup_img_hash_paths = {
        let mut map = BTreeMap::new();

        for (hash, path) in img_hashes.into_iter() {
            (*map.entry(hash).or_insert(vec![])).push(path)
        }
        map.retain(|_, vec| vec.len() > 1);
        map
    };

    println!();

    symlink_err_files(&args.output_path, err_img_paths.lock().unwrap().as_slice())?;
    symlink_dup_files(
        &args.output_path,
        dup_img_hash_paths
            .iter()
            .map(|x| (x.0.as_ref(), x.1.as_slice())),
    )?;

    ().wrap_ok()
}
