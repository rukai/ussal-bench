#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

use ussal_shared::{BenchMeasurement, BenchResult, BenchRun};

/// `cargo bench`
/// flags:
/// --ci        - writes new file to `bench_ci_history/benchn.cbor`, combine all of `bench_logs/` into `bench_ci_history_combined.cbor`
/// --file-name - write output to specified filename
/// by default overwrites bench.cbor in cwd
///
///
/// `cargo benchcompare`
/// either an xtask or an always backwards compatible `cargo-benchcompare`.
/// I have to be backwards compatible anyway for CI purposes, so may as well just do a `cargo-benchcompare`
/// usage: `cargo benchcompare $file1.cbor $file2.cbor`
/// compares changes between $file1.cbor and $file2.cbor

fn main() {
    let results = BenchRun::new(
        "Ussal Example Benchmarks".to_owned(),
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
    results.save("name.cbor");
}
