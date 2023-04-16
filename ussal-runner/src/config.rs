use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub tokens: Vec<String>,
}

impl OrchestratorConfig {
    fn path() -> PathBuf {
        config_path().join("config.json")
    }

    pub fn load() -> Self {
        let path = OrchestratorConfig::path();
        if path.exists() {
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
        } else {
            OrchestratorConfig { tokens: vec![] }
        }
    }

    #[allow(dead_code)]
    pub fn save(&self) {
        std::fs::write(
            OrchestratorConfig::path(),
            serde_json::to_vec(self).unwrap(),
        )
        .unwrap();
    }
}

pub fn config_path() -> PathBuf {
    dirs_next::config_dir().unwrap().join("UssalRunner")
}
