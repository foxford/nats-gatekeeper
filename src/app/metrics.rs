use chrono::Duration;
use once_cell::sync::Lazy;
use prometheus::{register_histogram, Histogram};

pub trait AuthorizeMetrics {
    fn measure(self) -> Self;
}

impl AuthorizeMetrics for Result<Duration, svc_authz::error::Error> {
    fn measure(self) -> Self {
        if let Ok(Ok(d)) = self.as_ref().map(|d| d.to_std()) {
            let nanos = f64::from(d.subsec_nanos()) / 1e9;
            METRICS.authz_time.observe(d.as_secs() as f64 + nanos)
        }
        self
    }
}

static METRICS: Lazy<Metrics> = Lazy::new(Metrics::new);

struct Metrics {
    authz_time: Histogram,
}

impl Metrics {
    fn new() -> Self {
        Metrics {
            authz_time: register_histogram!("auth_time", "Authorization time")
                .expect("Bad authz hist"),
        }
    }
}
