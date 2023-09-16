use std::collections::HashSet;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub ignore: Ignore
}

#[derive(Deserialize)]
pub struct Ignore {
    pub abs_path: HashSet<String>,
    pub regex: HashSet<String>
}
