use axum::{routing::get, Router};
use clap::Parser;
use config::OrchestratorConfig;
use std::net::{Ipv6Addr, SocketAddr};

mod cli;
mod config;
mod letsencrypt;
mod status_page;

#[tokio::main]
async fn main() {
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());
    tracing_subscriber::fmt().with_writer(non_blocking).init();

    let args = cli::Args::parse();
    let _config = OrchestratorConfig::load();
    let acceptor = letsencrypt::acme(&args).await;

    let app = Router::new().route("/", get(status_page::show_status));

    let addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, args.port));

    tracing::info!("Starting HTTPS on port: {}", args.port);
    axum_server::bind(addr)
        .acceptor(acceptor)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
