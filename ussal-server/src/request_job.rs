use crate::AppState;
use axum::extract::ws::WebSocket;
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use std::sync::Arc;

pub async fn request_job(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|stream| run_websocket(stream, state))
}

async fn run_websocket(stream: WebSocket, state: Arc<AppState>) {
    // let (tx, mut rx) = ussal_networking::axum::spawn_read_write_tasks::<
    //     runner_proto::JobRequest,
    //     runner_proto::JobResponse,
    // >(stream)
    // .await;
    // tx.send(runner_proto::JobRequest {
    //     job_id: Uuid::new_v4(),
    //     binary: vec![],
    //     ty: runner_proto::JobRequestType::ListBenches,
    // })
    // .unwrap();

    // while let Some(response) = rx.recv().await {
    //     tracing::info!("request_job received response: {:?}", response);
    // }
    match &state.handler {
        crate::job_handler::HandlerState::Orchestrator(orch) => {
            orch.connection_tx.send(stream).unwrap()
        }
        crate::job_handler::HandlerState::OrchestratorAndRunner => {
            panic!("external runners are not supported on OrchestratorAndRunner")
        }
    }
}
