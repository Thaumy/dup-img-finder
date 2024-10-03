use std::collections::BTreeSet;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::ops::Not;
use std::path::PathBuf;

use crate::infra::WrapResult;
use anyhow::{anyhow, Result};
use home::home_dir;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub size: u32,
    pub cache: String,
    pub ignore: Ignore,
}

#[derive(Deserialize)]
pub struct Ignore {
    pub abs_path: BTreeSet<String>,
    pub regex: BTreeSet<String>,
}

const DEFAULT_CFG: &str = r#"size=8
cache = "~/.config/dup-img-finder/cache.sqlite"

[ignore]
abs_path = []
regex = []
"#;

fn get_cfg_path() -> Result<PathBuf> {
    let home_path = home_dir().ok_or_else(|| anyhow!("Can not get home dir"))?;

    Ok(home_path
        .join(".config")
        .join("dup-img-finder")
        .join("cfg.toml"))
}

impl Config {
    pub fn read() -> Result<Self> {
        let cfg_path = get_cfg_path()?;

        if cfg_path.exists().not() {
            let mut f = File::create(cfg_path.clone())?;
            let _ = f.write(DEFAULT_CFG.as_ref())?;
        }

        let cfg_path = fs::read_to_string(cfg_path)?;

        toml::from_str::<Self>(&cfg_path)
            .expect("Failed to parse config")
            .wrap_ok()
    }
}
