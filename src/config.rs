use std::net::SocketAddr;

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
