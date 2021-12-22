use std::net::SocketAddr;
use std::time::Duration;

use serde::Deserialize;
use svc_authn::{
    jose::{Algorithm, ConfigMap},
    AccountId,
};
use svc_authz::ConfigMap as Authz;
use svc_error::extension::sentry::Config as SentryConfig;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub id: AccountId,
    pub authn: ConfigMap,
    pub authz: Authz,
    pub http_addr: SocketAddr,
    pub sentry: Option<SentryConfig>,
    pub metrics: Option<MetricsConfig>,
    #[serde(default = "default_max_payload")]
    pub max_payload: i64,
    #[serde(default = "default_max_subscriptions")]
    pub max_subscriptions: i64,
    #[serde(default = "default_expiration", with = "humantime_serde")]
    pub expiration: std::time::Duration,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MetricsConfig {
    pub http: MetricsHttpConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MetricsHttpConfig {
    pub bind_address: std::net::SocketAddr,
}

#[derive(Clone, Debug, Deserialize)]
pub struct JwtConfig {
    #[serde(deserialize_with = "svc_authn::serde::algorithm")]
    pub algorithm: Algorithm,
    #[serde(deserialize_with = "svc_authn::serde::file")]
    pub key: Vec<u8>,
}

pub(crate) fn load() -> Result<Config, config::ConfigError> {
    let mut parser = config::Config::default();
    parser.merge(config::File::with_name("App"))?;
    parser.merge(config::Environment::with_prefix("APP").separator("__"))?;
    parser.try_into::<Config>()
}

fn default_max_payload() -> i64 {
    100_000
}

fn default_max_subscriptions() -> i64 {
    5
}

fn default_expiration() -> Duration {
    Duration::from_secs(300)
}
