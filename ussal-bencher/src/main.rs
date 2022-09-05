#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

use ussal_shared::{BenchMeasurement, BenchResult, BenchRun};

fn main() {
    let results = BenchRun::new(
        "name".to_owned(),
        vec![BenchResult {
            name: "CoolBench".to_owned(),
            measurements: vec![
                BenchMeasurement {
                    measurement_name: "instructions".to_owned(),
                    unit: "I".to_owned(),
                    value: 0.0,
                },
                BenchMeasurement {
                    measurement_name: "walltime".to_owned(),
                    unit: "S".to_owned(),
                    value: 0.0,
                },
            ],
        }],
    );
    results.save();
}
