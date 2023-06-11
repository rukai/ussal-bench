use crate::cli::Args;
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::path::PathBuf;

// TODO: one day this should become kdl and these comments should go in the example config
#[derive(Deserialize, Debug)]
pub struct Config {
    /// The title used on the bench result web viewer
    pub title: String,
    /// The ussal orchestrator websocket endpoint
    pub address: String,
    pub runs: Vec<ConfigRun>,
    /// DANGER: Increment this number to force the CI bench history to reset.
    /// Provided in this strange form to allow reseting state stored in github pages.
    pub reset_ci_history: u32,
}

#[derive(Deserialize, Debug)]
pub struct ConfigRun {
    /// This run will be run only on runners which advertise a `machine type` matching this.
    pub machine_type: String,
    /// The target triple that this run should be compiled as
    pub target_triple: String,
}

impl Config {
    fn path(args: &Args) -> PathBuf {
        // TODO: get from crate root
        PathBuf::from(args.config_path.as_deref().unwrap_or("ussal.json"))
    }

    pub fn load(args: &Args) -> Result<Self> {
        let path = Config::path(args);
        if path.exists() {
            serde_json::from_slice(&std::fs::read(path)?).map_err(|e| anyhow!(e))
        } else {
            Err(anyhow!("{path:?} not yet setup in crate root."))
        }
    }
}
