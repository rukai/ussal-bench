use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// TODO: autoreload when file changes
#[derive(Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub tokens: Vec<Uuid>,
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
            let config = OrchestratorConfig {
                tokens: vec![Uuid::new_v4()],
            };
            config.save();
            config
        }
    }

    fn save(&self) {
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
