#![warn(clippy::all, clippy::nursery, clippy::cargo_common_metadata)]

mod args;
mod calc_img_hash;
mod find_img;
mod read_img;

use std::collections::HashMap;
use std::ops::ControlFlow;
use std::os::unix::fs::symlink as unix_symlink;
use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::{env, fs, thread, vec};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use crossbeam::queue::SegQueue;

use crate::args::Args;
use crate::calc_img_hash::calc_img_hash;
use crate::find_img::find_img;

fn main() -> Result<()> {
    let args: Args = Args::parse();

    let input_path: String = args.input_path;
    let output_path: String = args.output_path;
    let threads: usize = args
        .threads
        .unwrap_or_else(num_cpus::get);

    let img_paths = Arc::new(SegQueue::new());
    let (img_hash_result_tx, img_hash_result_rx) = channel();

    find_img(&img_paths, Path::new(&input_path))?;

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

    let mut err_count = 0_usize;
    if !err_img_paths.is_empty() {
        fs::create_dir_all(format!("{}/err", output_path))?;
        println!("{} Image format errors:", "[ERR]".red());
        err_img_paths
            .iter()
            .for_each(|path| {
                println!("{}", path);

                if let Err(e) = unix_symlink(
                    path.as_str(),
                    format!("{}/err/{}", output_path, err_count)
                ) {
                    println!(
                        "{} Failed to create symlink for: {} [{}]",
                        "[ERR]".red(),
                        path,
                        e
                    );
                }

                err_count += 1;
            });

        println!();
        println!(
            "{} Error image symlinks was created in: {}/err",
            "[INFO]".green(),
            env::current_dir()?.display()
        );
        println!();
    }

    let mut group_mark = '░';
    let count_align = dup_img_hash_paths
        .len()
        .to_string()
        .len();
    let mut dup_count = 0_usize;
    if !dup_img_hash_paths.is_empty() {
        fs::create_dir_all(format!("{}/dup", output_path))?;
        println!("{} Duplicate images:", "[DUP]".yellow());
        dup_img_hash_paths
            .iter()
            .filter(|(_, vec)| vec.len() > 1)
            .try_for_each(|(hash, vec)| {
                vec.iter().for_each(|path| {
                    println!("{dup_count:>count_align$} {group_mark} {}", path);
                    let file_name = path.split('/').last().unwrap();

                    if let Err(e) = unix_symlink(
                        path.as_str(),
                        format!("{}/dup/{}-{}-{}", output_path, base64_url::encode(hash), dup_count, file_name),
                    ) {
                        println!(
                            "{dup_count:>count_align$} {group_mark} {} Failed to create symlink for: {} [{}]",
                            "[ERR]".red(),
                            path,
                            e
                        );
                    }
                    dup_count += 1;
                });

                if group_mark == '▓' {
                    group_mark = '░'
                } else {
                    group_mark = '▓'
                }

                ControlFlow::<()>::Continue(())
            });

        println!();
        println!(
            "{} Duplicate image symlinks was created in: {}/dup",
            "[INFO]".green(),
            env::current_dir()?.display()
        );
    } else {
        println!("No duplicate images found");
    }

    Ok(())
}
