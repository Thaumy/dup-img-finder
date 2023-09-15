use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::ops::Not;

use anyhow::{anyhow, Result};
use home::home_dir;

pub fn read_rc() -> Result<HashSet<String>> {
    let home_path =
        home_dir().ok_or_else(|| anyhow!("Can not get home dir"))?;

    let rc_path = home_path.join(".difrc");

    if rc_path.exists().not() {
        File::create(rc_path.clone())?;
    }

    let rc_content = fs::read_to_string(rc_path)?;

    let ignore_paths = rc_content
        .split('\n')
        .map(|str| str.to_owned())
        .collect::<HashSet<String>>();

    Ok(ignore_paths)
}
