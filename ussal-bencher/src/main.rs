#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

use ussal_shared::{BenchMeasurement, BenchResult, BenchRun};

/// `cargo bench`
/// flags:
/// --ci
///     1. CI infrastructure runs: `git fetch origin/gh-pages; git checkout origin/gh-pages -- bench_ci_web_root`
///     2. CI infrastructure runs: `cargo bench --ci` which will:
///         1. writes new file to `bench_ci_web_root/history/benchn.cbor` # TODO: how do I persist this folder?
///         2. combine all of `bench_ci_web_root/history/` into `bench_ci_web_root/benches.cbor`
///         3. Generates viewer html + wasm into `bench_ci_web_root/`
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
    // TODO: OH I should just name with iso date lol
    // TODO: handle unwraps
    std::fs::create_dir_all("bench_ci_history").unwrap();
    let max = std::fs::read_dir("bench_ci_history")
        .unwrap()
        .filter_map(|dir_entry| {
            let name = dir_entry.unwrap().file_name();
            let name = name.to_str().unwrap();
            let number_portion = name.strip_prefix("bench")?.strip_suffix(".cbor")?;
            number_portion.parse().ok()
        })
        .max()
        .unwrap_or(0);
    results.save(&format!("bench_ci_history/bench{}.cbor", max + 1));

    // TODO: huh, I wonder if it would be easier to just read, process and then write the bench_ci_history_combined.cbor file directly without storing all the intermediate files.
    // Would be cheaper on disk space...
    // I probably could just do that considering I have --ci flag
}
