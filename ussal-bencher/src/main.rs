#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

use ussal_shared::{BenchMeasurement, BenchResult, BenchRun};

fn main() {
    let results = BenchRun::new(
        "name".to_owned(),
        vec![
            BenchResult {
                name: "CoolBench".to_owned(),
                measurements: vec![
                    BenchMeasurement {
                        name: "instructions".to_owned(),
                        unit: "I".to_owned(),
                        value: 1.0,
                    },
                    BenchMeasurement {
                        name: "walltime".to_owned(),
                        unit: "S".to_owned(),
                        value: 1.2,
                    },
                ],
            },
            BenchResult {
                name: "SadBench".to_owned(),
                measurements: vec![
                    BenchMeasurement {
                        name: "instructions".to_owned(),
                        unit: "I".to_owned(),
                        value: 10000.0,
                    },
                    BenchMeasurement {
                        name: "walltime".to_owned(),
                        unit: "S".to_owned(),
                        value: 10.0,
                    },
                ],
            },
        ],
    );
    results.save();
}
