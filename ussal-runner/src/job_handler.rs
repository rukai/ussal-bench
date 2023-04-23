use axum::extract::ws::Message;
use axum::extract::{ws::WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures::stream::StreamExt;
use futures::SinkExt;

use ussal_shared::runner_protocol::{JobRequest, JobResponse, JobResult};

pub async fn run_job(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(run_job_websocket)
}

async fn run_job_websocket(stream: WebSocket) {
    let state = State::OrchestratorAndRunner; // TODO: get from axum
    let (mut tx, mut rx) = stream.split();

    let config = crate::config::OrchestratorConfig::load();

    while let Some(Ok(message)) = rx.next().await {
        if let Message::Binary(binary) = &message {
            let request: serde_cbor::Result<JobRequest> = serde_cbor::from_slice(binary);
            match &request {
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
                    let response = match &state {
                        State::Orchestrator(orchestrator) => {
                            // TODO: maybe combine these calls? or maybe keep seperate for locking reasons?
                            orchestrator.send_request(request).await;

                            orchestrator.receive_response(request).await
                        }
                        State::OrchestratorAndRunner => crate::runner::run_job_request(request),
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

enum State {
    #[allow(dead_code)]
    Orchestrator(OrchestratorState),
    OrchestratorAndRunner,
}

struct OrchestratorState {}
impl OrchestratorState {
    /// pop a connection off the list of available connections
    async fn send_request(&self, _request: &JobRequest) {
        // Filter connections by request.os and request.arch
        //connection.send(message).await.unwrap();
        todo!()
    }

    /// pop a connection off the list of available connections
    async fn receive_response(&self, _request: &JobRequest) -> JobResponse {
        // Get response by matching the job_id
        todo!()
    }
}
