use anyhow::{anyhow, Context, Result};
use notify::{Config, RecommendedWatcher, Watcher};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::sync::watch::{channel, Receiver, Ref};
use uuid::Uuid;

use crate::cli::Args;

pub struct ReloadableOrchestratorConfig {
    pub config: Receiver<OrchestratorConfig>,
    _watcher: RecommendedWatcher,
}

impl ReloadableOrchestratorConfig {
    pub fn load(args: &Args) -> Self {
        let path = OrchestratorConfig::path(args);
        let (tx, config) = channel(
            OrchestratorConfig::load(&path)
                .context(format!("Failed to load {:?}", path))
                .unwrap(),
        );
        let path_clone = path.clone();
        let mut watcher = RecommendedWatcher::new(
            move |_| match OrchestratorConfig::load(&path_clone) {
                Ok(x) => {
                    tracing::info!("Succesfully reloaded {:?}", path_clone);
                    tx.send(x).unwrap();
                }
                Err(err) => {
                    tracing::error!("Failed to reload {:?}: {err:?}", path_clone)
                }
            },
            Config::default(),
        )
        .unwrap();
        watcher
            .watch(&path, notify::RecursiveMode::Recursive)
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
    fn path(args: &Args) -> PathBuf {
        match &args.config_path {
            Some(config_path) => Path::new(config_path).join("config.json"),
            None => default_config_path().join("config.json"),
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            serde_json::from_slice(&std::fs::read(path)?).map_err(|e| anyhow!(e))
        } else {
            let config = OrchestratorConfig {
                tokens: vec![Uuid::new_v4()],
            };
            config.save(path);
            Ok(config)
        }
    }

    fn save(&self, path: &Path) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, serde_json::to_vec(self).unwrap())
            .map_err(|e| anyhow::anyhow!("Failed to write to {path:?} {e}"))
            .unwrap();
    }
}

pub fn default_config_path() -> PathBuf {
    dirs_next::config_dir().unwrap().join("UssalRunner")
}
