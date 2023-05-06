use crate::AppState;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures::stream::{SplitSink, StreamExt};
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
                        process_request(&mut tx, request, &state).await;
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
            Message::Close(_) | Message::Ping(_) | Message::Pong(_) => {
                // we can just ignore these
            }
        }
    }
}

async fn fail_job(
    tx: &mut SplitSink<WebSocket, Message>,
    request: &orch_proto::JobRequest,
    message: String,
) {
    let response = orch_proto::JobResponse {
        job_id: request.job_id,
        result: orch_proto::JobResult::JobError(message),
    };
    tx.send(Message::Binary(serde_cbor::to_vec(&response).unwrap()))
        .await
        .unwrap();
}

async fn process_request(
    tx: &mut SplitSink<WebSocket, Message>,
    request: &orch_proto::JobRequest,
    state: &AppState,
) {
    if !state.config.borrow().tokens.contains(&request.auth_token) {
        fail_job(tx, request, "Invalid auth token".to_owned()).await;
        return;
    }
    let list_request = runner_proto::JobRequest {
        job_id: request.job_id,
        binary: request.binary.clone(),
        ty: runner_proto::JobRequestType::ListBenches,
    };
    let job_response = state.handler.run_job_request(list_request).await;
    let benches = job_response.ty.get_list_benches().unwrap();

    for bench in benches {
        let request = runner_proto::JobRequest {
            job_id: request.job_id,
            binary: request.binary.clone(),
            ty: runner_proto::JobRequestType::RunBench {
                bench_name: bench.clone(),
            },
        };
        let job_response = state.handler.run_job_request(request).await;
        let response = orch_proto::JobResponse {
            job_id: job_response.job_id,
            result: job_response
                .ty
                .get_run_bench()
                .map(|x| {
                    orch_proto::JobResult::BenchComplete(orch_proto::BenchComplete {
                        bench_name: bench.clone(),
                        wall_time: x.wall_time,
                    })
                })
                .unwrap_or_else(orch_proto::JobResult::BenchError),
        };
        tx.send(Message::Binary(serde_cbor::to_vec(&response).unwrap()))
            .await
            .unwrap();
    }

    let response = orch_proto::JobResponse {
        job_id: request.job_id,
        result: orch_proto::JobResult::JobComplete,
    };
    tx.send(Message::Binary(serde_cbor::to_vec(&response).unwrap()))
        .await
        .unwrap();
}

pub enum HandlerState {
    #[allow(dead_code)]
    Orchestrator(OrchestratorState),
    OrchestratorAndRunner,
}

impl HandlerState {
    async fn run_job_request(
        &self,
        request: runner_proto::JobRequest,
    ) -> runner_proto::JobResponse {
        match self {
            HandlerState::Orchestrator(_) => todo!(),
            HandlerState::OrchestratorAndRunner => {
                tokio::task::spawn_blocking(move || crate::runner::run_job_request(&request))
                    .await
                    .unwrap()
            }
        }
    }
}

pub struct OrchestratorState {}
impl OrchestratorState {
    /// pop a connection off the list of available connections
    async fn _send_request(&self, _request: &runner_proto::JobRequest) {
        // Filter connections by request.os and request.arch
        //connection.send(message).await.unwrap();
        todo!()
    }

    /// pop a connection off the list of available connections
    async fn _receive_response(
        &self,
        _request: &runner_proto::JobRequest,
    ) -> runner_proto::JobResponse {
        // Get response by matching the job_id
        todo!()
    }
}
