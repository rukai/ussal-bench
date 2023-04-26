use axum::routing::get;
use axum::Router;
use clap::Parser;
use cli::{Args, Mode};
use config::OrchestratorConfig;
use std::net::{Ipv6Addr, SocketAddr};

mod cli;
mod config;
mod install;
mod job_handler;
mod letsencrypt;
mod runner;
mod status_page;
mod system;

#[tokio::main]
async fn main() {
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());
    tracing_subscriber::fmt().with_writer(non_blocking).init();

    let args = Args::parse();
    match args.mode {
        Mode::Runner => todo!("Implement runner"),
        Mode::Orchestrator => orchestrator(args).await,
        Mode::OrchestratorAndRunner => orchestrator(args).await,
        Mode::DestructivelyInstallRunner => install::install_runner(args),
    }
}

async fn orchestrator(args: Args) {
    let _config = OrchestratorConfig::load();

    let app = Router::new()
        .route("/", get(status_page::show_status))
        //.route("/request_job", get(job_handler::request_job)) // turn connections into websocket, store websocket in state
        .route("/run_job", get(job_handler::run_job));

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
