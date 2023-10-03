use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::Path;

use anyhow::Result;
use colored::Colorize;
use regex::Regex;

fn is_img_ext(ext: OsString) -> bool {
    let supported_format = ["png", "jpg", "jpeg", "webp", "bmp", "gif"];
    supported_format
        .into_iter()
        .any(|format| ext == OsStr::new(format))
}

pub fn find_img(
    img_paths: &mut BTreeSet<String>,
    root_path: &Path,
    ignore_abs_paths: &BTreeSet<String>,
    ignore_path_regexes: &Vec<Regex>,
) -> Result<()> {
    if ignore_abs_paths.contains(root_path.to_str().unwrap()) {
        return Ok(());
    }
    if ignore_path_regexes
        .iter()
        .any(|r| r.is_match(root_path.to_str().unwrap()))
    {
        return Ok(());
    }

    // ignore symlink
    if root_path.is_symlink() {
        return Ok(());
    }

    for entry in fs::read_dir(root_path)? {
        let path = entry?.path();
        // ignore symlink
        if path.is_symlink() {
            continue;
        }

        if path
            .extension()
            .map(|ext| ext.to_ascii_lowercase())
            .is_some_and(is_img_ext)
        {
            println!(
                "{} {} {}",
                "[PATH]".yellow(),
                img_paths.len(),
                path.display()
            );
            img_paths.insert(format!("{}", path.display()));
        } else if path.is_dir() {
            find_img(
                img_paths,
                path.as_path(),
                ignore_abs_paths,
                ignore_path_regexes,
            )?;
        }
    }

    Ok(())
}
