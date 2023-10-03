use anyhow::Result;
use colored::Colorize;
use std::fs;

use crate::fmt_path_for_display::fmt_path_for_display;

pub fn read_file(percent: usize, img_path: &str) -> Result<Vec<u8>> {
    let display_path = fmt_path_for_display(img_path, 12);
    println!("{} {:>3}% {}", "[READ]".blue(), percent, display_path);

    let bytes = fs::read(img_path)?;
    Ok(bytes)
}
