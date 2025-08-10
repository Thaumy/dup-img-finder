use std::fs;
use std::ops::ControlFlow;
use std::os::unix::fs::{symlink as unix_symlink, MetadataExt};
use std::path::Path;

use anyhow::Result;
use colored::Colorize;

pub fn symlink_dup_files<'t>(
    output_path: &str,
    dup_img_hash_paths: impl ExactSizeIterator<Item = (&'t [u8], &'t mut [String])>,
) -> Result<()> {
    let mut group_mark = '░';
    let count_align = dup_img_hash_paths.len().to_string().len();
    let mut dup_count = 0_usize;

    if dup_img_hash_paths.len() == 0 {
        println!("No duplicate images found");
        return Ok(());
    }

    fs::create_dir_all(format!("{}/dup", output_path))?;
    println!("{} Duplicate images:", "[DUP]".yellow());

    dup_img_hash_paths
        .filter(|(_, vec)| vec.len() > 1)
        .try_for_each(|(hash, slice)| {
            slice.sort_by(|a, b| {
                let meta_a = fs::metadata(a).expect("Failed to get metadata");
                let meta_b = fs::metadata(b).expect("Failed to get metadata");
                let size_a = meta_a.size();
                let size_b = meta_b.size();
                let mtime_a = meta_a.mtime();
                let mtime_b = meta_b.mtime();
                if size_a == size_b {
                    mtime_a.cmp(&mtime_b).reverse()
                } else {
                    size_a.cmp(&size_b)
                }
            });

            slice.iter().for_each(|path| {
                println!("{dup_count:>count_align$} {group_mark} {}", path);

                let file_ext = Path::new(path)
                    .extension()
                    .expect("Failed to get file extension")
                    .to_string_lossy();

                if let Err(e) = unix_symlink(
                    path,
                    format!("{}/dup/{}-{}.{}", output_path, base64_url::encode(hash), dup_count, file_ext),
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
        "{} Duplicate image symlinks was created in: \n{}/dup",
        "[INFO]".green(),
        output_path
    );

    Ok(())
}
