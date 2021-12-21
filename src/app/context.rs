use std::sync::Arc;

use svc_authz::ClientMap as Authz;

use crate::config::Config;

pub trait GlobalContext: Sync {
    fn authz(&self) -> &Authz;
    fn config(&self) -> &Config;
    fn nats_key(&self) -> &str;
}

#[derive(Clone)]
pub struct AppContext {
    inner: Arc<Context>,
}

impl AppContext {
    pub fn new(config: Config, authz: Authz, nats_key: String) -> Self {
        Self {
            inner: Arc::new(Context {
                config,
                authz,
                nats_key,
            }),
        }
    }
}

impl GlobalContext for AppContext {
    fn authz(&self) -> &Authz {
        &self.inner.authz
    }

    fn config(&self) -> &Config {
        &self.inner.config
    }

    fn nats_key(&self) -> &str {
        &self.inner.nats_key
    }
}

#[derive(Clone)]
struct Context {
    config: Config,
    authz: Authz,
    nats_key: String,
}
