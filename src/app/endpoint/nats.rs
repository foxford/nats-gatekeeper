use anyhow::Context;
use axum::extract::{Extension, Path};
use serde::Serialize;
use std::time::SystemTime;
use svc_authn::AccountId;
use svc_authz::Authenticable;
use svc_utils::extractors::AuthnExtractor;
use uuid::Uuid;

use crate::app::context::{AppContext, GlobalContext};
use crate::app::error::{Error as AppError, ErrorExt, ErrorKind};
use crate::app::{authz::AuthzObject, metrics::AuthorizeMetrics};

#[derive(Serialize)]
struct TokenResponse {
    token: String,
    expires_in: u64,
}

pub async fn create_token(
    Extension(ctx): Extension<AppContext>,
    AuthnExtractor(agent_id): AuthnExtractor,
    Path((audience, classroom_id)): Path<(String, Uuid)>,
) -> Result<hyper::Response<hyper::Body>, AppError> {
    let classrooms = "classrooms";
    let object = AuthzObject::new(&[classrooms, &classroom_id.to_string(), "nats"]).into();

    ctx.authz()
        .authorize(
            audience,
            agent_id.as_account_id().to_owned(),
            object,
            "connect".into(),
        )
        .await
        .measure()?;

    let token = build_token(&ctx, classroom_id, agent_id.as_account_id())?;
    let body = serde_json::to_string(&token).unwrap();

    Ok(hyper::Response::builder()
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(hyper::Body::from(body))
        .unwrap())
}

fn build_token(
    ctx: &AppContext,
    classroom_id: Uuid,
    account_id: &AccountId,
) -> Result<TokenResponse, AppError> {
    let account_keypair = nats_jwt::KeyPair::from_seed(ctx.nats_key())
        .map_err(|e| {
            tracing::error!(error = ?e, "Failed to create kp");
            e
        })
        .context("Failed to create keypair from seed")
        .error(ErrorKind::InternalServerError)?;

    let user_keypair = nats_jwt::KeyPair::new_user();

    let allowed_topic = format!("classrooms.{classroom_id}.unreliable");
    // agent.{account_id}.{request, response}
    let request_wildcard = format!("agent.*.request.{classroom_id}");
    let response_wildcard = format!("agent.*.response.{classroom_id}.*.*");
    let request_topic = format!("agent.{}.request.{classroom_id}", account_id.label());
    let response_topic = format!("agent.{}.response.{classroom_id}.*.*", account_id.label());

    let user_token =
        nats_jwt::Token::new_user(account_keypair.public_key(), user_keypair.public_key())
            .bearer_token(true)
            .name(account_id.label())
            .max_payload(ctx.config().max_payload)
            .max_subscriptions(ctx.config().max_subscriptions)
            .allow_publish(&allowed_topic)
            .allow_subscribe(&allowed_topic)
            // allow to publish requests & replies
            .allow_publish(request_wildcard)
            .allow_publish(response_wildcard)
            // allow to read requests
            .allow_subscribe(request_topic)
            // allow to read replies
            .allow_subscribe(response_topic)
            .expires(expiration(ctx))
            .sign(&account_keypair);

    Ok(TokenResponse {
        token: user_token,
        expires_in: ctx.config().expiration.as_secs(),
    })
}

fn expiration(ctx: &AppContext) -> i64 {
    let duration = ctx.config().expiration;

    (SystemTime::now() + duration)
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
