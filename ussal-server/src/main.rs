use crate::system::{init_shutdown_handler, init_tracing};
use axum::routing::get;
use axum::Router;
use clap::Parser;
use cli::{Args, Mode};
use config::ReloadableOrchestratorConfig;
use job_handler::{HandlerState, OrchestratorState};
use std::net::{Ipv6Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::Semaphore;

mod cli;
mod config;
mod connection_assigner;
mod install;
mod job_handler;
mod letsencrypt;
mod request_job;
mod runner;
mod status_page;
mod system;
mod tracing_panic_handler;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let _tracing = init_tracing(args.log_format);
    let mut trigger_shutdown_rx = init_shutdown_handler().await;

    tokio::select! {
        _ = run(args) => {}
        _ = trigger_shutdown_rx.changed() => {}
    }
}

async fn run(args: Args) {
    match &args.mode {
        Mode::Runner { address } => runner::runner(address).await,
        Mode::Orchestrator { .. } => orchestrator(args, false).await,
        Mode::OrchestratorAndRunner { .. } => orchestrator(args, true).await,
        Mode::DestructivelyInstallRunner { .. } => install::install_runner(args),
    }
}

async fn orchestrator(args: Args, runner: bool) {
    let app = Router::new()
        .route("/", get(status_page::show_status))
        .route("/request_job", get(request_job::request_job))
        .route("/run_job", get(job_handler::run_job))
        .with_state(Arc::new(AppState::new(if runner {
            HandlerState::OrchestratorAndRunner(Semaphore::new(1))
        } else {
            let (request_tx, request_rx) = unbounded_channel();
            let (connection_tx, connection_rx) = unbounded_channel();
            tokio::spawn(connection_assigner::task(request_rx, connection_rx));
            HandlerState::Orchestrator(OrchestratorState::new(request_tx, connection_tx))
        })));

    let args = args.mode.orchestrator_args();

    let port = args
        .port
        .unwrap_or(if args.disable_https { 8000 } else { 443 });
    let addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, port));

    if args.disable_https {
        tracing::info!("Starting HTTP on port: {}", port);
        axum_server::bind(addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    } else {
        tracing::info!("Starting HTTPS on port: {}", port);
        axum_server::bind(addr)
            .acceptor(letsencrypt::acme(&args).await)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}

pub struct AppState {
    handler: HandlerState,
    config: ReloadableOrchestratorConfig,
}

impl AppState {
    fn new(handler: HandlerState) -> Self {
        AppState {
            handler,
            config: ReloadableOrchestratorConfig::load(),
        }
    }
}
