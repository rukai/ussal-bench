use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub address: String,
    pub runs: Vec<ConfigRun>,
}

#[derive(Deserialize, Debug)]
pub struct ConfigRun {
    pub machine_type: String,
    pub target_triple: String,
}

impl Config {
    fn path() -> PathBuf {
        // TODO: get from crate root
        PathBuf::from("ussal-config.json")
    }

    pub fn load() -> Result<Self> {
        let path = Config::path();
        if path.exists() {
            serde_json::from_slice(&std::fs::read(path)?).map_err(|e| anyhow!(e))
        } else {
            Err(anyhow!("ussal-config.json not yet setup in crate root."))
        }
    }
}
