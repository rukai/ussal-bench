use crate::cli::Args;
use rustls::ServerConfig;
use rustls_acme::axum::AxumAcceptor;
use rustls_acme::caches::DirCache;
use rustls_acme::AcmeConfig;
use std::sync::Arc;
use tokio_stream::StreamExt;

pub async fn acme(args: &Args) -> AxumAcceptor {
    let release = env!("PROFILE") == "release";

    let mut state = AcmeConfig::new(args.domains.clone())
        .cache_option(Some(DirCache::new(
            crate::config::config_path().join("acme_cache"),
        )))
        .contact(args.email.iter().map(|e| format!("mailto:{}", e)))
        .directory_lets_encrypt(release)
        .state();
    let rustls_config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_cert_resolver(state.resolver());
    let acceptor = state.axum_acceptor(Arc::new(rustls_config));

    tokio::spawn(async move {
        loop {
            match state.next().await.unwrap() {
                Ok(ok) => tracing::info!("event: {:?}", ok),
                Err(err) => tracing::error!("error: {:?}", err),
            }
        }
    });
    acceptor
}
