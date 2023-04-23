use crate::cli::Args;
use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use ussal_shared::orchestrator_protocol::{BenchComplete, JobRequest, JobResponse};
use uuid::Uuid;

type JobResults = HashMap<Uuid, JobResult>;

#[derive(Debug)]
pub struct JobResult {
    finished: bool,
    benches: Vec<BenchComplete>,
}

pub async fn run_jobs(args: Args, jobs: Vec<JobRequest>) -> Result<JobResults> {
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
                        ussal_shared::orchestrator_protocol::JobResult::BenchComplete(x) => {
                            if let Some(job) = job_results.get_mut(&response.job_id) {
                                job.benches.push(x);
                            }
                        }
                        ussal_shared::orchestrator_protocol::JobResult::BenchError(e) => {
                            // TODO: Fail only bench
                            return Err(anyhow!(e));
                        }
                        ussal_shared::orchestrator_protocol::JobResult::JobComplete => {
                            if let Some(job) = job_results.get_mut(&response.job_id) {
                                job.finished = true;
                            }
                        }
                        ussal_shared::orchestrator_protocol::JobResult::JobError(e) => {
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
