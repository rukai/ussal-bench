#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

pub mod runner_protocol;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct BenchArchive {
    version: i64,
    pub name: String,
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
            name,
            benches,
        }
    }

    pub fn load(name: &str) -> Self {
        serde_cbor::from_slice(&std::fs::read(name).unwrap()).unwrap()
    }

    pub fn load_from_cbor(bytes: &[u8]) -> Self {
        serde_cbor::from_slice(bytes).unwrap()
    }

    pub fn save(&self, name: &str) {
        std::fs::write(name, serde_cbor::to_vec(self).unwrap()).unwrap();
    }
}
