use std::sync::Arc;

use axum::{
    body::HttpBody,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use http::{Method, Request, Response};
use hyper::Body;
use svc_utils::middleware::MeteredRoute;
use tower_http::trace::TraceLayer;
use tracing::{
    error,
    field::{self, Empty},
    info, Span,
};

use super::{
    context::{AppContext, GlobalContext},
    endpoint,
    error::Error as AppError,
};

pub fn build_router(context: AppContext) -> Router {
    let authn = context.config().authn.clone();
    let router = Router::new()
        .metered_route(
            "/audiences/:audience/classrooms/:classroom_id/tokens",
            post(endpoint::nats::create_token).options(endpoint::read_options),
        )
        .layer(svc_utils::middleware::CorsLayer)
        .layer(Extension(context))
        .layer(Extension(Arc::new(authn)));

    let routes = Router::new().nest("/api/v1", router);

    let pingz_router = Router::new().route(
        "/healthz",
        get(|| async { Response::builder().body(Body::from("pong")).unwrap() }),
    );

    let routes = routes.merge(pingz_router);

    routes.layer(svc_utils::middleware::LogLayer::new()).layer(
        TraceLayer::new_for_http()
            .make_span_with(|request: &Request<Body>| {
                let header_as_field =
                    |name: &str| request.headers().get(name).and_then(|v| v.to_str().ok());

                let span = tracing::error_span!(
                    "http-api-request",
                    path = request.uri().path(),
                    query = request.uri().query(),
                    method = %request.method(),
                    status_code = Empty,
                    kind = Empty,
                    detail = Empty,
                    body_size = Empty,
                    ulms_app_audience = header_as_field("ulms-app-audience"),
                    ulms_app_label = header_as_field("ulms-app-label"),
                    ulms_app_version = header_as_field("ulms-app-version"),
                    ulms_scope = header_as_field("ulms-scope"),
                );

                if request.method() != Method::GET && request.method() != Method::OPTIONS {
                    span.record(
                        "body_size",
                        &field::debug(request.body().size_hint().upper()),
                    );
                }

                span
            })
            .on_response(
                |response: &Response<_>, latency: std::time::Duration, span: &Span| {
                    span.record("status_code", &field::debug(response.status()));
                    if response.status().is_success() {
                        info!("response generated in {:?}", latency);
                    } else {
                        error!("response generated in {:?}", latency);
                    }
                },
            ),
    )
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        self.notify_sentry();

        (self.status(), Json(self.to_svc_error())).into_response()
    }
}
