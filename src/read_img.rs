use colored::Colorize;
use image::{DynamicImage, ImageResult};

#[inline]
pub fn read_img(img_path: &String) -> ImageResult<DynamicImage> {
    println!("{} {}", "[READ]".green(), img_path);
    image::open(img_path.as_str())
}
