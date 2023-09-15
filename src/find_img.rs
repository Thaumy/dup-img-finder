use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use colored::Colorize;
use crossbeam::queue::SegQueue;

#[inline]
fn is_img_ext(ext: OsString) -> bool {
    let supported_format =
        ["png", "jpg", "jpeg", "webp", "bmp", "gif"];
    supported_format
        .into_iter()
        .any(|format| ext == OsStr::new(format))
}

pub fn find_img(
    img_paths: &Arc<SegQueue<String>>,
    root_path: &Path,
    ignore_paths: &HashSet<String>
) -> Result<()> {
    if ignore_paths.contains(root_path.to_str().unwrap()) {
        return Ok(());
    }

    for entry in fs::read_dir(root_path)? {
        let path = entry?.path();

        if path
            .extension()
            .map(|ext| ext.to_ascii_lowercase())
            .is_some_and(is_img_ext)
        {
            println!("{} {}", "[PATH]".yellow(), path.display());
            img_paths.push(format!("{}", path.display()))
        } else if path.is_dir() {
            find_img(img_paths, path.as_path(), ignore_paths)?;
        }
    }

    Ok(())
}
