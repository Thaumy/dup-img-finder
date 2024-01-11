use std::fs;
use std::ops::ControlFlow;
use std::os::unix::fs::symlink as unix_symlink;

use crate::infra::WrapResult;
use anyhow::Result;
use colored::Colorize;

pub fn symlink_dup_files<'t>(
    output_path: &str,
    dup_img_hash_paths: impl ExactSizeIterator<Item = (&'t [u8], &'t [String])>,
) -> Result<()> {
    let mut group_mark = '░';
    let count_align = dup_img_hash_paths.len().to_string().len();
    let mut dup_count = 0_usize;

    if dup_img_hash_paths.len() == 0 {
        println!("No duplicate images found");
        return ().wrap_ok();
    }

    fs::create_dir_all(format!("{}/dup", output_path))?;
    println!("{} Duplicate images:", "[DUP]".yellow());
    dup_img_hash_paths
        .filter(|(_, vec)| vec.len() > 1)
        .try_for_each(|(hash, vec)| {
            vec.iter().for_each(|path| {
                println!("{dup_count:>count_align$} {group_mark} {}", path);
                let file_name = path.split('/').last().expect("Failed to get file name");

                if let Err(e) = unix_symlink(
                    path,
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
        "{} Duplicate image symlinks was created in: \n{}/dup",
        "[INFO]".green(),
        output_path
    );

    ().wrap_ok()
}
