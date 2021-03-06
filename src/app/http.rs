use std::sync::Arc;

use axum::{
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use http::Response;
use hyper::Body;
use svc_utils::middleware::MeteredRoute;

use super::{
    context::{AppContext, GlobalContext},
    endpoint,
    error::Error as AppError,
};

pub fn build_router(context: AppContext) -> Router {
    let authn = context.config().authn.clone();
    let router = Router::new()
        // TODO: Delete it in the next release
        .metered_route(
            "/audiences/:audience/scopes/:scope/tokens",
            post(endpoint::nats::create_token_for_scope).options(endpoint::read_options),
        )
        .metered_route(
            "/audiences/:audience/classrooms/:classroom_id/tokens",
            post(endpoint::nats::create_token_for_classroom).options(endpoint::read_options),
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

    routes.layer(svc_utils::middleware::LogLayer::new())
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        self.notify_sentry();

        (self.status(), Json(self.to_svc_error())).into_response()
    }
}
