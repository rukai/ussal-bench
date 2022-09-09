#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

use ussal_shared::{BenchMeasurement, BenchResult, BenchRun};

/// `cargo bench`
/// flags:
/// --ci
///     1. CI infrastructure runs: `git fetch origin/gh-pages; git checkout origin/gh-pages -- bench_ci_web_root`
///     2. CI infrastructure runs: `cargo bench --ci` which will:
///         1. read existing state from bench_ci_web_root/bench_history.cbor
///         2. insert new run to state
///         3. write new state to bench_ci_web_root/bench_history.cbor
///         4. Generates viewer html + wasm into `bench_ci_web_root/`
///     3. CI infrastructure then needs to:
///        on main branch checkin: copy bench_ci_web_root to gh-pages/bench_ci_web_root
///        on PR branch received: copy bench_ci_web_root to gh-pages/repo_name/branch_name
///     
/// --file-name
///     - write output to specified filename
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
    // TODO: handle unwraps
    std::fs::create_dir_all("bench_ci_web_root").unwrap();
    results.save("bench_ci_web_root/bench_history.cbor");
    results.save("bench.cbor");

    // TODO: build viewer as wasm if it exists
    //       else fallback to undecided method (either: pull from github release, include_bytes it, download from crates.io and build from source)

    // TODO: include_bytes!("viewer.wasm");
    // TODO: how to generate viewer.wasm before publish??

    std::fs::write("bench_ci_web_root/index.html", include_bytes!("index.html")).unwrap();
}
