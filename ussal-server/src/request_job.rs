use crate::connection_assigner::Connection;
use crate::AppState;
use axum::extract::ws::WebSocket;
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use std::sync::Arc;
use ussal_shared::runner_protocol::{JobResponse, JobResponseType};

pub async fn request_job(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|stream| run_websocket(stream, state))
}

async fn run_websocket(stream: WebSocket, state: Arc<AppState>) {
    match &state.handler {
        crate::job_handler::HandlerState::Orchestrator(orch) => {
            let (tx, mut rx) = ussal_networking::axum::spawn_read_write_tasks(stream).await;
            let machine_type = match rx.recv().await {
                Some(JobResponse {
                    ty: JobResponseType::Handshake { machine_type },
                    ..
                }) => machine_type,
                Some(x) => {
                    tracing::error!("Expected handshake but was {x:?}");
                    return;
                }
                None => {
                    tracing::error!("Expected handshake but no message was received");
                    return;
                }
            };
            orch.connection_tx
                .send(Connection {
                    tx,
                    rx,
                    machine_type,
                })
                .unwrap();
        }
        crate::job_handler::HandlerState::OrchestratorAndRunner(_) => {
            panic!("external runners are not supported on OrchestratorAndRunner")
        }
    }
}
