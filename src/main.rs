use anyhow::Result;
use std::collections::HashMap;

use colored::Colorize;
use crossbeam::queue::SegQueue;
use image_hasher::HasherConfig;
use std::ffi::OsStr;
use std::ops::ControlFlow;
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::{env, fs, thread, vec};

fn find_img(img_paths: &Arc<SegQueue<String>>, root_path: &Path) -> Result<()> {
    for entry in fs::read_dir(root_path)? {
        let path = entry?.path();

        if path
            .extension()
            .map(|ext| ext.to_ascii_lowercase())
            .is_some_and(|ext| ext == OsStr::new("png") || ext == OsStr::new("jpg"))
        {
            img_paths.push(format!("{}", path.display()))
        } else if path.is_dir() {
            find_img(img_paths, path.as_path())?;
        }
    }

    Ok(())
}

fn calc_img_hash(img_path: &String, img_hash_result_tx: &Sender<Result<(String, String), String>>) {
    let hasher = HasherConfig::new().to_hasher();

    let result = match image::open(img_path) {
        Ok(img) => {
            let hash = base64_url::encode(hasher.hash_image(&img).as_bytes());
            println!("{} {} {}", "[CALC]".cyan(), hash, img_path);
            Ok((hash, img_path.clone()))
        }
        Err(e) => {
            println!(
                "{} Failed to open image: {} [{}]",
                "[ERR]".red(),
                img_path,
                e
            );
            Err(img_path.clone())
        }
    };

    img_hash_result_tx.send(result).unwrap()
}

fn main() -> Result<()> {
    let root_path = {
        let mut args = env::args();
        args.next();
        args.next().unwrap()
    };

    let img_paths = Arc::new(SegQueue::new());
    let (img_hash_result_tx, img_hash_result_rx) = channel();

    find_img(&img_paths, Path::new(&root_path))?;

    let mut workers = vec![];
    for _ in 0..num_cpus::get() {
        let img_paths = img_paths.clone();
        let img_hash_result_tx = img_hash_result_tx.clone();
        let worker = thread::spawn(move || {
            while let Some(img_path) = img_paths.pop() {
                calc_img_hash(&img_path, &img_hash_result_tx);
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
                Ok((hash, path)) => (*img_hash_map.entry(hash).or_insert(vec![])).push(path),
                Err(msg) => err_img_paths.push(msg),
            }
        }
        img_hash_map.retain(|_, vec| vec.len() > 1);

        (img_hash_map, err_img_paths)
    };
    workers.into_iter().for_each(|worker| {
        worker.join().unwrap();
    });

    println!();

    let mut err_count = 0_usize;
    if !err_img_paths.is_empty() {
        fs::create_dir_all("err")?;
        println!("{} Image format errors:", "[ERR]".red());
        err_img_paths.iter().for_each(|path| {
            println!("{}", path);

            if let Err(e) = std::os::unix::fs::symlink(path, format!("err/{}", err_count)) {
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
    let count_align = dup_img_hash_paths.len().to_string().len();
    let mut dup_count = 0_usize;
    if !dup_img_hash_paths.is_empty() {
        fs::create_dir_all("dup")?;
        println!("{} Duplicate images:", "[DUP]".yellow());
        dup_img_hash_paths
            .iter()
            .filter(|(_, vec)| vec.len() > 1)
            .try_for_each(|(hash, vec)| {
                vec.iter().for_each(|path| {
                    println!("{dup_count:>count_align$} {group_mark} {}", path);

                    if let Err(e) =
                        std::os::unix::fs::symlink(path, format!("dup/{}-{}", hash, dup_count))
                    {
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
