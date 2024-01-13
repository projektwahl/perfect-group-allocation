use std::convert::Infallible;

use perfect_group_allocation_database::DatabaseError;
use perfect_group_allocation_openidconnect::error::OpenIdConnectError;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("header error: {0}")]
    Header(#[from] headers::Error),
    #[error("IO error: {0}")]
    File(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("webserver error: {0}")]
    Hyper(#[from] hyper::Error),
    #[error("unknown error: {0}")]
    Other(#[from] anyhow::Error),
    #[error("env var error: {0}")]
    EnvVar(#[from] std::env::VarError),
    #[error("rustls error: {0}")]
    Rustls(#[from] tokio_rustls::rustls::Error),
    #[error("poison error: {0}")]
    Poison(#[from] std::sync::PoisonError<()>),
    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    // #[cfg(feature = "perfect-group-allocation-telemetry")]
    //#[error("trace error: {0}")]
    //Trace(#[from] TraceError),
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("wrong csrf token")]
    WrongCsrfToken,
    #[error("no accept remaining")]
    NoAcceptRemaining,
    #[error(
        "The request session is still held onto. Maybe you keep it alive inside a streaming \
         response?"
    )]
    SessionStillHeld,
    #[error("openid connect error: {0}")]
    OpenIdConnect(#[from] OpenIdConnectError),
    #[error(
        "Höchstwahrscheinlich ist deine Anmeldesession abgelaufen und du musst es erneut \
         versuchen. Wenn dies wieder auftritt, melde das Problem bitte an einen \
         Serveradministrator."
    )]
    OpenIdTokenNotFound,
    #[error("Der Serveradministrator hat OpenID nicht konfiguriert.")]
    OpenIdNotConfigured,
}

impl From<Infallible> for AppError {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

impl From<diesel_async::pooled_connection::deadpool::PoolError> for AppError {
    fn from(value: diesel_async::pooled_connection::deadpool::PoolError) -> Self {
        AppError::Database(value.into())
    }
}
