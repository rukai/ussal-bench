use crate::{cli::Args, config::Config};
use anyhow::{anyhow, Result};
use std::{collections::HashMap, time::Duration};
use tokio::time::timeout;
use tokio_tungstenite::connect_async;
use ussal_shared::orchestrator_protocol::{BenchComplete, JobRequest};

#[derive(Debug)]
pub struct JobResult {
    finished: bool,
    benches: Vec<BenchComplete>,
}

pub async fn run_jobs(
    args: &Args,
    config: Config,
    jobs: Vec<JobRequest>,
) -> Result<impl Iterator<Item = BenchComplete>> {
    assert!(!jobs.is_empty(), "jobs must contain values otherwise we will deadlock waiting for a response that will never come");
    let mut job_results = HashMap::new();

    let uri = args.address.as_ref().unwrap_or(&config.address);
    let (ws_stream, _) = timeout(Duration::from_secs(10), connect_async(uri))
        .await
        .map_err(|_| anyhow!("Timed out connecting to {uri} after 10 seconds"))?
        .map_err(|e| anyhow!(e).context(format!("Failed to connect to {uri}")))?;
    tracing::info!("WebSocket handshake has been successfully completed");
    let (tx, mut rx) = ussal_networking::spawn_read_write_tasks::<
        ussal_shared::orchestrator_protocol::JobRequest,
        ussal_shared::orchestrator_protocol::JobResponse,
    >(ws_stream)
    .await;

    for job in jobs {
        job_results.insert(
            job.job_id,
            JobResult {
                finished: false,
                benches: vec![],
            },
        );
        tx.send(job).unwrap();
    }

    while let Some(response) = rx.recv().await {
        match response.result {
            ussal_shared::orchestrator_protocol::JobResult::BenchComplete(bench) => {
                if let Some(job) = job_results.get_mut(&response.job_id) {
                    tracing::info!("{:?}", bench);
                    job.benches.push(bench);
                } else {
                    return Err(anyhow!("BenchComplete contained unknown job_id"));
                }
            }
            ussal_shared::orchestrator_protocol::JobResult::BenchError(e) => {
                // TODO: Fail only bench
                return Err(anyhow!(e));
            }
            ussal_shared::orchestrator_protocol::JobResult::JobComplete => {
                if let Some(job) = job_results.get_mut(&response.job_id) {
                    job.finished = true;
                } else {
                    return Err(anyhow!("JobComplete contained unknown job_id"));
                }
            }
            ussal_shared::orchestrator_protocol::JobResult::JobError(e) => return Err(anyhow!(e)),
        }
        if job_results.values().all(|x| x.finished) {
            return Ok(job_results
                .into_values()
                .flat_map(|x| x.benches.into_iter()));
        }
    }

    Err(anyhow!(
        "Connection was closed before all jobs were finished. Last known job state: {job_results:#?}"
    ))
}
