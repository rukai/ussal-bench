use crate::cli::SandboxMode;
use crate::connection_assigner::{Connection, Request};
use crate::AppState;
use axum::extract::ws::WebSocket;
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures::future::join_all;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{oneshot, Semaphore};
use ussal_networking::orchestrator_protocol as orch_proto;
use ussal_networking::runner_protocol as runner_proto;

pub async fn run_job(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|stream| run_job_websocket(stream, state))
}

async fn run_job_websocket(stream: WebSocket, state: Arc<AppState>) {
    let (mut tx, mut rx) = ussal_networking::axum::spawn_read_write_tasks::<
        orch_proto::JobResponse,
        orch_proto::JobRequest,
    >(stream)
    .await;

    while let Some(request) = rx.recv().await {
        process_request(&mut tx, &request, &state).await;
    }
}

async fn fail_job(
    tx: &mut UnboundedSender<orch_proto::JobResponse>,
    request: &orch_proto::JobRequest,
    message: String,
) {
    let response = orch_proto::JobResponse {
        job_id: request.job_id,
        result: orch_proto::JobResult::JobError(message),
    };
    tx.send(response).unwrap();
}

async fn process_request(
    tx: &mut UnboundedSender<orch_proto::JobResponse>,
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
        ty: runner_proto::JobRequestType::ListBenchesCriterion,
    };
    let job_response = state
        .handler
        .run_job_request(list_request, &request.machine_type)
        .await;
    let benches = job_response.ty.get_list_benches().unwrap();

    let run = benches.iter().map(|bench| {
        let tx = tx.clone();
        async move {
            let machine_type = &request.machine_type;
            let request = runner_proto::JobRequest {
                job_id: request.job_id,
                binary: request.binary.clone(),
                ty: runner_proto::JobRequestType::RunBenchCriterion {
                    bench_name: bench.clone(),
                },
            };
            let job_response = state.handler.run_job_request(request, machine_type).await;
            let response = orch_proto::JobResponse {
                job_id: job_response.job_id,
                result: job_response
                    .ty
                    .get_run_bench()
                    .map(|x| {
                        orch_proto::JobResult::BenchComplete(orch_proto::BenchComplete {
                            bench_name: bench.clone(),
                            keys: [("type".to_owned(), "walltime (ns)".to_owned())]
                                .into_iter()
                                .collect(),
                            wall_time: x.wall_time,
                        })
                    })
                    .unwrap_or_else(orch_proto::JobResult::BenchError),
            };
            tx.send(response).unwrap();
        }
    });
    join_all(run).await;

    let response = orch_proto::JobResponse {
        job_id: request.job_id,
        result: orch_proto::JobResult::JobComplete,
    };
    tx.send(response).unwrap();
}

pub enum HandlerState {
    #[allow(dead_code)]
    Orchestrator(OrchestratorState),
    OrchestratorAndRunner {
        semaphore: Semaphore,
        sandbox_mode: SandboxMode,
    },
}

impl HandlerState {
    async fn run_job_request(
        &self,
        request: runner_proto::JobRequest,
        machine_type: &str,
    ) -> runner_proto::JobResponse {
        match self {
            HandlerState::Orchestrator(state) => loop {
                let mut connection = state.get_connection(machine_type).await;
                connection
                    .tx
                    .send(request.clone())
                    .expect("assigner task should never die");
                match connection.rx.recv().await {
                    Some(response) => return response,
                    None => {
                        tracing::error!("Connection to runner was lost before it sent a response")
                    }
                }
            },
            HandlerState::OrchestratorAndRunner {
                sandbox_mode,
                semaphore,
            } => {
                let _permit = semaphore.acquire().await.unwrap();
                let sandbox_mode = *sandbox_mode;
                tokio::task::spawn_blocking(move || {
                    crate::runner::run_job_request(sandbox_mode, &request)
                })
                .await
                .unwrap()
            }
        }
    }
}

pub struct OrchestratorState {
    request_tx: UnboundedSender<Request>,
    pub connection_tx: UnboundedSender<Connection>,
}

impl OrchestratorState {
    pub fn new(
        request_tx: UnboundedSender<Request>,
        connection_tx: UnboundedSender<Connection>,
    ) -> OrchestratorState {
        OrchestratorState {
            request_tx,
            connection_tx,
        }
    }

    /// pop a connection off the list of available connections
    async fn get_connection(&self, machine_type: &str) -> Connection {
        // Filter connections by request.os and request.arch
        let (tx, rx) = oneshot::channel();
        self.request_tx
            .send(Request {
                machine_type: machine_type.to_owned(),
                tx,
            })
            .unwrap();

        rx.await.unwrap()
    }
}
