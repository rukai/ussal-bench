#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

pub mod orchestrator_protocol;
pub mod runner_protocol;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct BenchArchive {
    version: u32,
    reset_id: u32,
    pub title: String,
    pub benches: Vec<Bench>,
}

#[derive(Serialize, Deserialize)]
pub struct Bench {
    pub name: String,
    // A set of keys used when combining measurements and for filtering
    // A hashmap is used rather than hardcoded fields because I expect all of them to be handled in the same way.
    // By default we will include:
    // * type -> instructions|walltime possibly include the unit in brackets on the walltime
    // * os -> macos|linux|windows
    // * arch -> x86|arm64
    // Users will also be able to overwrite these defaults and add their own
    pub keys: HashMap<String, String>,
    pub measurements: Vec<BenchMeasurement>,
}

#[derive(Serialize, Deserialize)]
pub struct BenchMeasurement {
    // TODO: build hash and/or datetime?
    pub value: f32,
}

impl BenchArchive {
    pub fn new(name: String, benches: Vec<Bench>) -> Self {
        BenchArchive {
            version: 0,
            reset_id: 0,
            title: name,
            benches,
        }
    }

    pub fn reset_if_mismatch(&mut self, reset_id: u32) {
        if reset_id != self.reset_id {
            self.benches.clear();
            self.reset_id = reset_id;
        }
    }

    pub fn insert(&mut self, new_archive: BenchArchive) {
        for new_bench in new_archive.benches {
            for bench in &mut self.benches {
                if bench.keys == new_bench.keys && bench.name == new_bench.name {
                    bench
                        .measurements
                        .extend(new_bench.measurements.into_iter());
                    break;
                }
            }
        }
    }

    pub fn load(path: &str) -> Result<Self> {
        // TODO: retain backwards compatibility by performing format upgrades here
        serde_cbor::from_slice(
            &std::fs::read(path)
                .map_err(|e| anyhow!(e).context(format!("Failed to read {path:?} from disk")))?,
        )
        .map_err(|e| anyhow!(e).context(format!("Failed to parse {path:?} as json")))
    }

    pub fn load_from_cbor(bytes: &[u8]) -> Self {
        serde_cbor::from_slice(bytes).unwrap()
    }

    pub fn save(&self, name: &str) {
        std::fs::write(name, serde_cbor::to_vec(self).unwrap()).unwrap();
    }
}
