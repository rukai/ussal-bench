#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

mod cli;
mod gen_web;

use anyhow::{anyhow, Result};
use clap::Parser;
use cli::Args;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use ussal_shared::{
    runner_protocol::{BenchComplete, JobRequest, JobResponse},
    Bench, BenchArchive, BenchMeasurement,
};
use uuid::Uuid;

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
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());
    tracing_subscriber::fmt().with_writer(non_blocking).init();

    let args = Args::parse();

    let jobs = get_jobs(&args);
    let results = run_jobs(args, jobs).await;
    tracing::info!("results: {:?}", results);

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
}

fn get_jobs(args: &Args) -> Vec<JobRequest> {
    vec![
        JobRequest {
            auth_token: args.auth_token,
            job_id: Uuid::new_v4(),
            binary: vec![],
            os: "linux".to_owned(),
            arch: "x86_64".to_owned(),
        },
        JobRequest {
            auth_token: args.auth_token,
            job_id: Uuid::new_v4(),
            binary: vec![],
            os: "linux".to_owned(),
            arch: "arm64".to_owned(),
        },
    ]
}

type JobResults = HashMap<Uuid, JobResult>;

#[derive(Debug)]
struct JobResult {
    finished: bool,
    benches: Vec<BenchComplete>,
}

async fn run_jobs(args: Args, jobs: Vec<JobRequest>) -> Result<JobResults> {
    let mut job_results = HashMap::new();

    let uri = format!("wss://{}/run_job", args.address);
    let (ws_stream, _) = connect_async(&uri)
        .await
        .map_err(|e| anyhow!(e).context(format!("Failed to connect to {uri}")))?;
    tracing::info!("WebSocket handshake has been successfully completed");
    let (mut tx, mut rx) = ws_stream.split();

    for job in jobs {
        job_results.insert(
            job.job_id,
            JobResult {
                finished: false,
                benches: vec![],
            },
        );
        tx.send(Message::Binary(serde_cbor::to_vec(&job).unwrap()))
            .await
            .unwrap();
    }

    while let Some(Ok(message)) = rx.next().await {
        if let Message::Binary(binary) = message {
            let response: serde_cbor::Result<JobResponse> = serde_cbor::from_slice(&binary);
            match response {
                Ok(response) => {
                    match response.result {
                        ussal_shared::runner_protocol::JobResult::BenchComplete(x) => {
                            if let Some(job) = job_results.get_mut(&response.job_id) {
                                job.benches.push(x);
                            }
                        }
                        ussal_shared::runner_protocol::JobResult::BenchError(e) => {
                            // TODO: Fail only bench
                            return Err(anyhow!(e));
                        }
                        ussal_shared::runner_protocol::JobResult::JobComplete => {
                            if let Some(job) = job_results.get_mut(&response.job_id) {
                                job.finished = true;
                            }
                        }
                        ussal_shared::runner_protocol::JobResult::JobError(e) => {
                            return Err(anyhow!(e))
                        }
                    }
                }
                Err(err) => {
                    return Err(anyhow!(
                        "Invalid cbor in JobResponse {err}\ncontents: {binary:?}"
                    ));
                }
            }
        }
        if job_results.values().all(|x| x.finished) {
            return Ok(job_results);
        }
    }

    Err(anyhow!(
        "Connection was closed before all jobs were finished"
    ))
}
