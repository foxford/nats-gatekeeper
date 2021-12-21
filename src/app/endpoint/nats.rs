use anyhow::Context;
use axum::extract::{Extension, Path};
use serde::Serialize;
use svc_authn::AccountId;
use svc_authz::Authenticable;
use svc_utils::extractors::AuthnExtractor;

use crate::app::context::{AppContext, GlobalContext};
use crate::app::error::{Error as AppError, ErrorExt, ErrorKind};
use crate::app::{authz::AuthzObject, metrics::AuthorizeMetrics};

#[derive(Serialize)]
struct TokenResponse {
    token: String,
}

pub async fn create_token(
    Extension(ctx): Extension<AppContext>,
    AuthnExtractor(agent_id): AuthnExtractor,
    Path((audience, scope)): Path<(String, String)>,
) -> Result<hyper::Response<hyper::Body>, AppError> {
    let object = AuthzObject::new(&["scopes", &scope, "nats"]).into();

    ctx.authz()
        .authorize(
            audience,
            agent_id.as_account_id().to_owned(),
            object,
            "connect".into(),
        )
        .await
        .measure()?;

    let token = build_token(&ctx, &scope, &agent_id.as_account_id())?;
    let body = serde_json::to_string(&token).unwrap();

    Ok(hyper::Response::builder()
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(hyper::Body::from(body))
        .unwrap())
}

fn build_token(
    ctx: &AppContext,
    scope: &str,
    account_id: &AccountId,
) -> Result<TokenResponse, AppError> {
    let account_keypair = nats_jwt::KeyPair::from_seed(ctx.nats_key())
        .context("Failed to create keypair from seed")
        .error(ErrorKind::AuthorizationFailed)?;

    let user_keypair = nats_jwt::KeyPair::new_user();

    let allowed_topic = format!("scope.{}.unreliable", scope);

    let user_token =
        nats_jwt::Token::new_user(account_keypair.public_key(), user_keypair.public_key())
            .bearer_token(true)
            .name(account_id.label())
            .max_payload(ctx.config().max_payload)
            .max_subscriptions(ctx.config().max_subscriptions)
            .allow_publish(allowed_topic.clone())
            .allow_subscribe(allowed_topic)
            .sign(&account_keypair);

    Ok(TokenResponse { token: user_token })
}
