use crate::AppState;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures::stream::StreamExt;
use futures::SinkExt;
use std::sync::Arc;
use ussal_shared::orchestrator_protocol as orch_proto;
use ussal_shared::runner_protocol as runner_proto;

pub async fn run_job(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|stream| run_job_websocket(stream, state))
}

async fn run_job_websocket(stream: WebSocket, state: Arc<AppState>) {
    let (mut tx, mut rx) = stream.split();

    while let Some(Ok(message)) = rx.next().await {
        match &message {
            Message::Binary(binary) => {
                let request: serde_cbor::Result<orch_proto::JobRequest> =
                    serde_cbor::from_slice(binary);
                match &request {
                    Ok(request) => {
                        if !state.config.tokens.contains(&request.auth_token) {
                            let response = orch_proto::JobResponse {
                                job_id: request.job_id,
                                result: orch_proto::JobResult::JobError(
                                    "Invalid auth token".to_owned(),
                                ),
                            };
                            tx.send(Message::Binary(serde_cbor::to_vec(&response).unwrap()))
                                .await
                                .unwrap();
                            return;
                        }
                        match &state.handler {
                            HandlerState::Orchestrator(orchestrator) => {
                                // TODO: maybe combine these calls? or maybe keep seperate for locking reasons?
                                let request = runner_proto::JobRequest {
                                    job_id: request.job_id,
                                    binary: request.binary.clone(),
                                    ty: runner_proto::JobRequestType::ListBenches,
                                };
                                orchestrator.send_request(&request).await;

                                orchestrator.receive_response(&request).await;
                                todo!()
                            }
                            HandlerState::OrchestratorAndRunner => {
                                let list_request = runner_proto::JobRequest {
                                    job_id: request.job_id,
                                    binary: request.binary.clone(),
                                    ty: runner_proto::JobRequestType::ListBenches,
                                };
                                let job_response = tokio::task::spawn_blocking(move || {
                                    crate::runner::run_job_request(&list_request)
                                })
                                .await
                                .unwrap();
                                let benches = job_response.ty.get_list_benches().unwrap();

                                for bench in benches {
                                    let request = runner_proto::JobRequest {
                                        job_id: request.job_id,
                                        binary: request.binary.clone(),
                                        ty: runner_proto::JobRequestType::RunBench {
                                            bench_name: bench.clone(),
                                        },
                                    };
                                    let job_response = tokio::task::spawn_blocking(move || {
                                        crate::runner::run_job_request(&request)
                                    })
                                    .await
                                    .unwrap();
                                    let response = orch_proto::JobResponse {
                                        job_id: job_response.job_id,
                                        result: job_response
                                            .ty
                                            .get_run_bench()
                                            .map(|x| {
                                                orch_proto::JobResult::BenchComplete(
                                                    orch_proto::BenchComplete {
                                                        bench_name: bench.clone(),
                                                        wall_time: x.wall_time,
                                                    },
                                                )
                                            })
                                            .unwrap_or_else(|err| {
                                                orch_proto::JobResult::BenchError(err)
                                            }),
                                    };
                                    tx.send(Message::Binary(
                                        serde_cbor::to_vec(&response).unwrap(),
                                    ))
                                    .await
                                    .unwrap();
                                }
                            }
                        }

                        let response = orch_proto::JobResponse {
                            job_id: request.job_id,
                            result: orch_proto::JobResult::JobComplete,
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
            Message::Text(text) => {
                tracing::error!("Message::Text is not supported in the ussal protocol, but a Message::Text was received: {text:?}");
                return;
            }
            Message::Close(close) => {
                tracing::error!("hmmm wonder what this is for: {close:?}");
                return;
            }
            Message::Ping(_) | Message::Pong(_) => { /* we can just ignore these */ }
        }
    }
}

pub enum HandlerState {
    #[allow(dead_code)]
    Orchestrator(OrchestratorState),
    OrchestratorAndRunner,
}

pub struct OrchestratorState {}
impl OrchestratorState {
    /// pop a connection off the list of available connections
    async fn send_request(&self, _request: &runner_proto::JobRequest) {
        // Filter connections by request.os and request.arch
        //connection.send(message).await.unwrap();
        todo!()
    }

    /// pop a connection off the list of available connections
    async fn receive_response(
        &self,
        _request: &runner_proto::JobRequest,
    ) -> runner_proto::JobResponse {
        // Get response by matching the job_id
        todo!()
    }
}
