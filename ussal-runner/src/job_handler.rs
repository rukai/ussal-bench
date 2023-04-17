use axum::extract::ws::Message;
use axum::extract::{ws::WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures::stream::StreamExt;
use futures::SinkExt;

use ussal_shared::runner_protocol::{BenchComplete, JobRequest, JobResponse, JobResult};

pub async fn run_job(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(run_job_websocket)
}

async fn run_job_websocket(stream: WebSocket) {
    let (mut tx, mut rx) = stream.split();

    let config = crate::config::OrchestratorConfig::load();

    while let Some(Ok(message)) = rx.next().await {
        if let Message::Binary(binary) = message {
            let request: serde_cbor::Result<JobRequest> = serde_cbor::from_slice(&binary);
            match request {
                Ok(request) => {
                    if !config.tokens.contains(&request.auth_token) {
                        let response = JobResponse {
                            job_id: request.job_id,
                            result: JobResult::JobError("Invalid auth token".to_owned()),
                        };
                        tx.send(Message::Binary(serde_cbor::to_vec(&response).unwrap()))
                            .await
                            .unwrap();
                        return;
                    }
                    tracing::info!("request {request:?}");

                    let response = JobResponse {
                        job_id: request.job_id,
                        result: JobResult::BenchComplete(BenchComplete {
                            bench_name: "CoolBench".to_owned(),
                            wall_time: 413.,
                        }),
                    };
                    tx.send(Message::Binary(serde_cbor::to_vec(&response).unwrap()))
                        .await
                        .unwrap();

                    let response = JobResponse {
                        job_id: request.job_id,
                        result: JobResult::JobComplete,
                    };
                    tx.send(Message::Binary(serde_cbor::to_vec(&response).unwrap()))
                        .await
                        .unwrap();
                }
                Err(err) => {
                    tracing::warn!("Invalid cbor in JobRequest {err}\ncontents: {binary:?}");
                    return;
                }
            }
        }
    }
}
