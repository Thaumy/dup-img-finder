use std::os::unix::fs::symlink as unix_symlink;
use std::{env, fs};

use crate::infra::WrapResult;
use anyhow::Result;
use colored::Colorize;

pub fn symlink_err_files(output_path: &str, err_img_paths: &[String]) -> Result<()> {
    let mut err_count = 0_usize;
    if err_img_paths.is_empty() {
        return ().wrap_ok();
    }
    fs::create_dir_all(format!("{}/err", output_path))?;
    println!("{} Image format errors:", "[ERR]".red());
    err_img_paths.iter().for_each(|path| {
        println!("{}", path);

        if let Err(e) = unix_symlink(path.as_str(), format!("{}/err/{}", output_path, err_count)) {
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

    ().wrap_ok()
}
