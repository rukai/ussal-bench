#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

mod cli;
mod config;
mod gen_web;
mod get_jobs;
mod run_jobs;

use clap::Parser;
use cli::Args;
use ussal_archive::{Bench, BenchArchive, BenchMeasurement};

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

    let config = match config::Config::load() {
        Ok(config) => config,
        Err(err) => {
            tracing::error!("Failed to load config: {err:?}");
            return 1;
        }
    };

    let jobs = match get_jobs::get_jobs(&args, &config) {
        Ok(jobs) => jobs,
        Err(err) => {
            tracing::error!("Failed to get benchmarks: {err}");
            return 1;
        }
    };
    let job_results = match run_jobs::run_jobs(&args, &config, jobs).await {
        Ok(results) => results,
        Err(err) => {
            tracing::error!("Failed to run remote benchmarks: {err}");
            return 1;
        }
    };

    let benches: Vec<Bench> = job_results
        .flat_map(|job| {
            let machine_type = job.machine_type;
            job.benches.into_iter().map(move |bench| Bench {
                name: bench.bench_name,
                keys: {
                    let mut keys = bench.keys;
                    keys.insert("machine".to_owned(), machine_type.clone());
                    keys
                },
                measurements: vec![BenchMeasurement {
                    value: bench.wall_time,
                }],
            })
        })
        .collect();

    let results = BenchArchive::new(config.title.to_owned(), benches);

    if args.ci {
        // TODO: handle unwraps
        gen_web::generate_web();
        let history = match BenchArchive::load("bench_ci_web_root/bench_history.cbor") {
            Ok(mut history) => {
                history.title = config.title;
                history.reset_if_mismatch(config.reset_ci_history);
                history.insert(results);
                history
            }
            Err(err) => {
                // TODO: file not existing should not be a warn
                tracing::warn!("Failed to load history, history is starting from scratch: {err:?}");
                results
            }
        };
        history.save("bench_ci_web_root/bench_history.cbor");
    } else {
        results.save("bench.cbor");
    }

    0
}
