use std::sync::Arc;

use http::StatusCode;
use svc_error::{extension::sentry, Error as SvcError};

struct ErrorKindProperties {
    status: StatusCode,
    kind: &'static str,
    title: &'static str,
    is_notify_sentry: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorKind {
    AccessDenied,
    AuthorizationFailed,
}

impl ErrorKind {
    pub fn status(self) -> StatusCode {
        let properties: ErrorKindProperties = self.into();
        properties.status
    }

    pub fn is_notify_sentry(self) -> bool {
        let properties: ErrorKindProperties = self.into();
        properties.is_notify_sentry
    }
}

impl From<ErrorKind> for ErrorKindProperties {
    fn from(val: ErrorKind) -> Self {
        match val {
            ErrorKind::AccessDenied => ErrorKindProperties {
                status: StatusCode::FORBIDDEN,
                kind: "access_denied",
                title: "Access denied",
                is_notify_sentry: false,
            },
            ErrorKind::AuthorizationFailed => ErrorKindProperties {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                kind: "authorization_failed",
                title: "Authorization failed",
                is_notify_sentry: false,
            },
        }
    }
}

pub struct Error {
    kind: ErrorKind,
    err: Arc<anyhow::Error>,
}

impl Error {
    pub fn new(kind: ErrorKind, err: anyhow::Error) -> Self {
        Self {
            kind,
            err: Arc::new(err),
        }
    }

    pub fn status(&self) -> StatusCode {
        self.kind.status()
    }

    pub fn to_svc_error(&self) -> SvcError {
        let properties: ErrorKindProperties = self.kind.into();

        SvcError::builder()
            .status(properties.status)
            .kind(properties.kind, properties.title)
            .detail(&self.err.to_string())
            .build()
    }

    pub fn notify_sentry(&self) {
        if !self.kind.is_notify_sentry() {
            return;
        }

        if let Err(e) = sentry::send(self.err.clone()) {
            tracing::error!("Failed to send error to sentry, reason = {:?}", e);
        }
    }
}

impl From<svc_authz::Error> for Error {
    fn from(source: svc_authz::Error) -> Self {
        let kind = match source.kind() {
            svc_authz::ErrorKind::Forbidden(_) => ErrorKind::AccessDenied,
            _ => ErrorKind::AuthorizationFailed,
        };

        Self {
            kind,
            err: Arc::new(source.into()),
        }
    }
}

pub trait ErrorExt<T> {
    fn error(self, kind: ErrorKind) -> Result<T, Error>;
}

impl<T> ErrorExt<T> for Result<T, anyhow::Error> {
    fn error(self, kind: ErrorKind) -> Result<T, Error> {
        self.map_err(|source| Error::new(kind, source))
    }
}
