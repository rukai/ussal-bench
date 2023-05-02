use anyhow::{anyhow, Context, Result};
use notify::{Config, RecommendedWatcher, Watcher};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::watch::{channel, Receiver, Ref};
use uuid::Uuid;

pub struct ReloadableOrchestratorConfig {
    pub config: Receiver<OrchestratorConfig>,
    _watcher: RecommendedWatcher,
}

impl ReloadableOrchestratorConfig {
    pub fn load() -> Self {
        let (tx, config) = channel(
            OrchestratorConfig::load()
                .context(format!("Failed to load {:?}", OrchestratorConfig::path()))
                .unwrap(),
        );

        let mut watcher = RecommendedWatcher::new(
            move |_| match OrchestratorConfig::load() {
                Ok(x) => {
                    tracing::info!("Succesfully reloaded {:?}", OrchestratorConfig::path());
                    tx.send(x).unwrap();
                }
                Err(err) => {
                    tracing::error!("Failed to reload {:?}: {err:?}", OrchestratorConfig::path())
                }
            },
            Config::default(),
        )
        .unwrap();
        watcher
            .watch(&config_path(), notify::RecursiveMode::Recursive)
            .unwrap();

        ReloadableOrchestratorConfig {
            config,
            _watcher: watcher,
        }
    }

    pub fn borrow(&self) -> Ref<'_, OrchestratorConfig> {
        self.config.borrow()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrchestratorConfig {
    pub tokens: Vec<Uuid>,
}

impl OrchestratorConfig {
    fn path() -> PathBuf {
        config_path().join("config.json")
    }

    pub fn load() -> Result<Self> {
        let path = OrchestratorConfig::path();
        if path.exists() {
            serde_json::from_slice(&std::fs::read(path)?).map_err(|e| anyhow!(e))
        } else {
            let config = OrchestratorConfig {
                tokens: vec![Uuid::new_v4()],
            };
            config.save();
            Ok(config)
        }
    }

    fn save(&self) {
        let path = OrchestratorConfig::path();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::to_vec(self).unwrap())
            .map_err(|e| anyhow::anyhow!("Failed to write to {path:?} {e}"))
            .unwrap();
    }
}

pub fn config_path() -> PathBuf {
    dirs_next::config_dir().unwrap().join("UssalRunner")
}
