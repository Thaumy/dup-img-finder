use anyhow::Result;
use std::collections::HashMap;

use colored::Colorize;
use crossbeam::queue::SegQueue;
use image_hasher::HasherConfig;
use std::ffi::OsStr;
use std::ops::ControlFlow;
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::{env, fs, thread, vec};

fn map_dir(img_paths: &Arc<SegQueue<String>>, path: &Path) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let path = entry?.path();

        if path
            .extension()
            .map(|ext| ext.to_ascii_lowercase())
            .is_some_and(|ext| ext == OsStr::new("png") || ext == OsStr::new("jpg"))
        {
            img_paths.push(format!("{}", path.display()))
        } else if path.is_dir() {
            map_dir(img_paths, path.as_path())?;
        }
    }

    Ok(())
}

fn calc_img_hash(
    img_path: &String,
    img_hash_path_chan_sender: &Sender<(String, String)>,
    img_err_paths: &Arc<Mutex<Vec<String>>>,
) {
    let hasher = HasherConfig::new().to_hasher();

    match image::open(img_path) {
        Ok(image) => {
            let hash = hasher.hash_image(&image).to_base64();
            println!("{} {} │ {}", "[CALC]".cyan(), hash, img_path);
            img_hash_path_chan_sender
                .send((hash, img_path.clone()))
                .unwrap();
        }
        Err(_) => img_err_paths.lock().unwrap().push(img_path.clone()),
    }
}

fn main() -> Result<()> {
    let mut args = env::args();
    args.next();
    let path_arg = args.next().unwrap();
    let path = Path::new(path_arg.as_str());

    let img_paths = Arc::new(SegQueue::new());
    let err_img_paths = Arc::new(Mutex::new(vec![]));
    let (img_hash_path_chan_sender, img_hash_chan_recv) = channel();

    map_dir(&img_paths, path)?;

    let mut workers = vec![];
    for _ in 0..num_cpus::get() {
        let img_paths = img_paths.clone();
        let err_img_paths = err_img_paths.clone();
        let img_hash_path_chan_sender = img_hash_path_chan_sender.clone();
        let worker = thread::spawn(move || {
            while let Some(img_path) = img_paths.pop() {
                calc_img_hash(&img_path, &img_hash_path_chan_sender, &err_img_paths);
            }
        });
        workers.push(worker);
    }
    drop(img_hash_path_chan_sender);

    let mut img_hash_map = HashMap::new();
    for (hash, path) in img_hash_chan_recv {
        (*img_hash_map.entry(hash).or_insert(vec![])).push(path);
    }
    let mut dup_img_hash_paths = HashMap::new();
    img_hash_map.iter().for_each(|(hash, vec)| {
        if vec.len() > 1 {
            dup_img_hash_paths.insert(hash, vec);
        }
    });
    for worker in workers {
        let _ = worker.join();
    }

    println!();

    let mut err_count = 0_usize;
    if !err_img_paths.lock().unwrap().is_empty() {
        fs::create_dir_all("err")?;
        println!("{} Image format errors:", "[ERR]".red());
        err_img_paths.lock().unwrap().iter().for_each(|path| {
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
    let mut dup_count = 0_usize;
    if !dup_img_hash_paths.is_empty() {
        fs::create_dir_all("dup")?;
        println!("{} Duplicate images:", "[DUP]".yellow());
        dup_img_hash_paths
            .iter()
            .filter(|(_, vec)| vec.len() > 1)
            .try_for_each(|(hash, vec)| {
                if let Err(e) = fs::create_dir_all(format!("dup/{}", hash)) {
                    println!("{} Failed to create dir: {} ({})", "[ERR]".red(), hash, e);
                    return ControlFlow::<()>::Continue(());
                }

                vec.iter().for_each(|path| {
                    println!("{group_mark} {}", path);

                    if let Err(e) =
                        std::os::unix::fs::symlink(path, format!("dup/{}/{}", hash, dup_count))
                    {
                        println!(
                            "{group_mark} {} Failed to create symlink for: {} [{}]",
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

                ControlFlow::Continue(())
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
