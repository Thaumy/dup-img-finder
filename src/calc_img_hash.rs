use anyhow::Result;
use image_hasher::Hasher;

use crate::infra::result::WrapResult;

#[allow(clippy::type_complexity)]
pub fn calc_img_hash(hasher: &Hasher, img_bytes: Result<Vec<u8>>) -> Result<Box<[u8]>> {
    let di = image::load_from_memory(&img_bytes?[..])?;
    let ih = hasher.hash_image(&di);
    let hash = Box::<[u8]>::from(ih.as_bytes());
    hash.wrap_ok()
}
