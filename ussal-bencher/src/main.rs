#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

mod cli;
mod gen_web;
mod get_jobs;
mod run_jobs;

use clap::Parser;
use cli::Args;
use std::collections::HashMap;
use ussal_shared::{Bench, BenchArchive, BenchMeasurement};

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

/// TODO:
/// current thoughts:
/// combine --ci and --file-name:
/// output format is combinable with other outputs.
/// output format embeds OS string
///
/// This way we can parallelize the workload across multiple workflows
///
/// If we have a combiner workflow copied in by default it should be easier to introduce a new parallel workflow
///
/// need to think through this approach.

#[tokio::main]
async fn main() {
    std::process::exit(run().await);
}

async fn run() -> i32 {
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());
    tracing_subscriber::fmt().with_writer(non_blocking).init();

    let args = Args::parse();

    let jobs = match get_jobs::get_jobs(&args) {
        Ok(jobs) => jobs,
        Err(err) => {
            tracing::error!("Failed to get benchmarks: {err}");
            return 1;
        }
    };
    let results = match run_jobs::run_jobs(args, jobs).await {
        Ok(results) => results,
        Err(err) => {
            tracing::error!("Failed to run remote benchmarks: {err}");
            return 1;
        }
    };

    tracing::info!("results: {:#?}", results);

    let results = BenchArchive::new(
        "Ussal Example Benchmarks".to_owned(),
        vec![
            Bench {
                name: "CoolBench".to_owned(),
                keys: HashMap::from([("type".to_owned(), "instructions".to_owned())]),
                measurements: vec![
                    BenchMeasurement { value: 1.0 },
                    BenchMeasurement { value: 1.2 },
                    BenchMeasurement { value: 1.3 },
                    BenchMeasurement { value: 1.7 },
                    BenchMeasurement { value: 1.5 },
                ],
            },
            Bench {
                name: "CoolBench".to_owned(),
                keys: HashMap::from([("type".to_owned(), "walltime".to_owned())]),
                measurements: vec![
                    BenchMeasurement { value: 1.0 },
                    BenchMeasurement { value: 0.8 },
                ],
            },
            Bench {
                name: "SadBench".to_owned(),
                keys: HashMap::from([("type".to_owned(), "instructions".to_owned())]),
                measurements: vec![
                    BenchMeasurement { value: 10000.0 },
                    BenchMeasurement { value: 10. },
                ],
            },
            Bench {
                name: "SadBench".to_owned(),
                keys: HashMap::from([("type".to_owned(), "walltime".to_owned())]),
                measurements: vec![
                    BenchMeasurement { value: 10.0 },
                    BenchMeasurement { value: 10.0 },
                    BenchMeasurement { value: 15. },
                ],
            },
        ],
    );

    // TODO: handle unwraps
    gen_web::generate_web();
    results.save("bench_ci_web_root/bench_history.cbor");

    results.save("bench.cbor");

    0
}
