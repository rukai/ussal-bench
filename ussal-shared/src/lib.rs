#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BenchRun {
    version: i64,
    name: String,
    pub results: Vec<BenchResult>,
}

#[derive(Serialize, Deserialize)]
pub struct BenchResult {
    pub name: String,
    pub measurements: Vec<BenchMeasurement>,
}

// Given that the programmer is free to change the bench at any time we should just do our best to display what we have.
// That means always including the metadata for each measurement.
// We should be able to compress the result with gzip or something to save a bunch of space.
#[derive(Serialize, Deserialize)]
pub struct BenchMeasurement {
    pub measurement_name: String,
    pub unit: String,
    pub value: f32,
}

impl BenchRun {
    pub fn new(name: String, results: Vec<BenchResult>) -> Self {
        BenchRun {
            version: 0,
            name,
            results,
        }
    }

    pub fn load(name: &str) -> Self {
        let name = format!("{}.cbor", name);
        serde_cbor::from_slice(&std::fs::read(name).unwrap()).unwrap()
    }

    pub fn save(&self) {
        let name = format!("{}.cbor", self.name);
        std::fs::write(name, serde_cbor::to_vec(self).unwrap()).unwrap();
    }
}
