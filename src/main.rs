use anyhow::Result;
use tracing::warn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP: &str = env!("CARGO_PKG_NAME");

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_log::LogTracer::init()?;

    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());
    let subscriber = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .json()
        .flatten_event(true);
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(subscriber);

    tracing::subscriber::set_global_default(subscriber)?;
    warn!("Launching {}, version: {}", APP, APP_VERSION);

    let authz_cache = redis::create_redis();
    app::run(authz_cache).await
}

mod app;
mod config;
mod redis;
