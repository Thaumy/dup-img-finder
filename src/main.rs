#![warn(clippy::all, clippy::nursery, clippy::cargo_common_metadata)]

mod args;
mod calc_img_hash;
mod cfg;
mod find_img;
mod fmt_path_for_display;
mod read_cfg;
mod read_img;
mod symlink_dup_files;
mod symlink_err_files;

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::{thread, vec};

use anyhow::Result;
use clap::Parser;
use crossbeam::queue::SegQueue;
use regex::Regex;

use crate::args::Args;
use crate::calc_img_hash::calc_img_hash;
use crate::find_img::find_img;
use crate::read_cfg::read_cfg;
use crate::symlink_dup_files::symlink_dup_files;
use crate::symlink_err_files::symlink_err_files;

fn main() -> Result<()> {
    let args: Args = Args::parse();

    let input_path: String = args.input_path;
    let output_path: String = args.output_path;
    let threads: usize = args
        .threads
        .unwrap_or_else(num_cpus::get);

    let img_paths = {
        let mut img_paths = HashSet::new();

        let cfg = read_cfg()?;
        let ignore_abs_paths = cfg.ignore.abs_path;
        let ignore_path_regexes = cfg
            .ignore
            .regex
            .into_iter()
            .map(|s| Regex::new(&s).unwrap())
            .collect();
        find_img(
            &mut img_paths,
            Path::new(&input_path),
            &ignore_abs_paths,
            &ignore_path_regexes
        )?;
        {
            let sq = img_paths.into_iter().fold(
                SegQueue::new(),
                |acc, it| {
                    acc.push(it);
                    acc
                }
            );
            Arc::new(sq)
        }
    };

    let (img_hash_result_tx, img_hash_result_rx) = channel();
    let total_img_count = img_paths.len() as f64;
    let calc_img_count = Arc::new(AtomicUsize::new(0));

    let mut workers = vec![];
    for _ in 0..threads {
        let img_paths = img_paths.clone();
        let img_hash_result_tx = img_hash_result_tx.clone();
        let calc_img_count = calc_img_count.clone();

        let worker = thread::spawn(move || {
            while let Some(img_path) = img_paths.pop() {
                let calc_img_count = calc_img_count
                    .fetch_add(1, Ordering::SeqCst)
                    as f64;
                let percent = (calc_img_count / total_img_count *
                    100.0)
                    .round() as usize;
                calc_img_hash(percent, img_path, &img_hash_result_tx);
            }
        });

        workers.push(worker);
    }
    drop(img_hash_result_tx);

    let (dup_img_hash_paths, err_img_paths) = {
        let mut err_img_paths = vec![];
        let mut img_hash_map = HashMap::new();

        for result in img_hash_result_rx {
            match result {
                Ok((hash, path)) => (*img_hash_map
                    .entry(hash)
                    .or_insert(vec![]))
                .push(path),
                Err(msg) => err_img_paths.push(msg)
            }
        }
        img_hash_map.retain(|_, vec| vec.len() > 1);

        (img_hash_map, err_img_paths)
    };
    workers
        .into_iter()
        .for_each(|worker| {
            worker.join().unwrap();
        });

    println!();

    symlink_err_files(&output_path, err_img_paths.as_slice())?;
    symlink_dup_files(&output_path, &dup_img_hash_paths)?;

    Ok(())
}
