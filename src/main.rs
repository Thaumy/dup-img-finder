#![warn(clippy::all, clippy::nursery, clippy::cargo_common_metadata)]

mod args;
mod calc_img_hash;
mod find_img;
mod read_img;
mod read_rc;
mod symlink_dup_files;
mod symlink_err_files;

use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::{thread, vec};

use anyhow::Result;
use clap::Parser;
use crossbeam::queue::SegQueue;

use crate::args::Args;
use crate::calc_img_hash::calc_img_hash;
use crate::find_img::find_img;
use crate::read_rc::read_rc;
use crate::symlink_dup_files::symlink_dup_files;
use crate::symlink_err_files::symlink_err_files;

fn main() -> Result<()> {
    let args: Args = Args::parse();

    let input_path: String = args.input_path;
    let output_path: String = args.output_path;
    let threads: usize = args
        .threads
        .unwrap_or_else(num_cpus::get);

    let img_paths = Arc::new(SegQueue::new());
    let (img_hash_result_tx, img_hash_result_rx) = channel();

    let ignore_paths = read_rc()?;
    find_img(&img_paths, Path::new(&input_path), &ignore_paths)?;

    let mut workers = vec![];
    for _ in 0..threads {
        let img_paths = img_paths.clone();
        let img_hash_result_tx = img_hash_result_tx.clone();
        let worker = thread::spawn(move || {
            while let Some(img_path) = img_paths.pop() {
                calc_img_hash(img_path, &img_hash_result_tx);
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
