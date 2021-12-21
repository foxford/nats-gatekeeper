use anyhow::{Context as AnyhowContext, Result};
use futures::StreamExt;
use signal_hook::consts::TERM_SIGNALS;
use svc_authz::cache::AuthzCache;
use tracing::{error, info};

use crate::app::context::AppContext;
use crate::app::http::build_router;
use crate::config;

pub(crate) async fn run(nats_key: String, authz_cache: Option<Box<dyn AuthzCache>>) -> Result<()> {
    let config = config::load().context("Failed to load config")?;
    info!("App config: {:?}", config);

    let authz = svc_authz::ClientMap::new(&config.id, authz_cache, config.authz.clone(), None)
        .context("Error converting authz config to clients")?;

    let context = AppContext::new(config.clone(), authz, nats_key);

    let metrics_task = config
        .metrics
        .as_ref()
        .map(|metrics| svc_utils::metrics::MetricsServer::new(metrics.http.bind_address));

    let (graceful_tx, graceful_rx) = tokio::sync::oneshot::channel();
    let http_task = tokio::spawn(
        axum::Server::bind(&config.http_addr)
            .serve(build_router(context).into_make_service())
            .with_graceful_shutdown(async move {
                let _ = graceful_rx.await;
            }),
    );

    let mut signals_stream = signal_hook_tokio::Signals::new(TERM_SIGNALS)?.fuse();
    let signals = signals_stream.next();

    let _ = signals.await;

    info!("Received signal, shutting down");

    let _ = graceful_tx.send(());

    if let Some(metrics_task) = metrics_task {
        metrics_task.shutdown().await;
    }

    if let Err(e) = http_task.await {
        error!("Failed to await http server completion, err = {:?}", e);
    }

    Ok(())
}

mod authz;
mod context;
mod endpoint;
mod error;
mod http;
mod metrics;
