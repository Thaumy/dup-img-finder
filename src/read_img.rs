use colored::Colorize;
use image::{DynamicImage, ImageResult};

use crate::fmt_path_for_display::fmt_path_for_display;

#[inline]
pub fn read_img(
    percent: usize,
    img_path: &str
) -> ImageResult<DynamicImage> {
    let display_path = fmt_path_for_display(img_path, 12);
    println!("{} {:>3}% {}", "[READ]".blue(), percent, display_path);
    image::open(img_path)
}
