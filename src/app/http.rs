use std::sync::Arc;

use axum::{
    response::IntoResponse,
    routing::{get, post},
    AddExtensionLayer, Router,
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
        .metered_route(
            "/tokens",
            post(endpoint::nats::create_token).options(endpoint::read_options),
        )
        .layer(svc_utils::middleware::CorsLayer)
        .layer(AddExtensionLayer::new(context))
        .layer(AddExtensionLayer::new(Arc::new(authn)));

    let routes = Router::new().nest("/api/v1", router);

    let pingz_router = Router::new().route(
        "/healthz",
        get(|| async { Response::builder().body(Body::from("pong")).unwrap() }),
    );

    let routes = routes.merge(pingz_router);

    routes.layer(svc_utils::middleware::LogLayer::new())
}

impl IntoResponse for AppError {
    type Body = axum::body::Body;

    type BodyError = <Self::Body as axum::body::HttpBody>::Error;

    fn into_response(self) -> hyper::Response<Self::Body> {
        self.notify_sentry();

        let err = self.to_svc_error();
        let error =
            serde_json::to_string(&err).unwrap_or_else(|_| "Failed to serialize error".to_string());
        http::Response::builder()
            .status(self.status())
            .body(axum::body::Body::from(error))
            .expect("This is a valid response")
    }
}
