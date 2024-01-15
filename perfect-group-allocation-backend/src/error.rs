use std::backtrace::Backtrace;
use std::convert::Infallible;
use std::fmt::{Debug, Display};

use bytes::Bytes;
use headers::ContentType;
use http::{Response, StatusCode};
use http_body_util::StreamBody;
use perfect_group_allocation_config::ConfigError;
use perfect_group_allocation_css::index_css;
use perfect_group_allocation_database::DatabaseError;
use perfect_group_allocation_openidconnect::error::OpenIdConnectError;
use zero_cost_templating::async_iterator_extension::AsyncIteratorStream;
use zero_cost_templating::Unsafe;

use crate::routes::error;
use crate::session::Session;
use crate::{yieldfi, yieldfv, ResponseTypedHeaderExt as _};

#[derive(thiserror::Error)]
pub enum AppError {
    #[error("header error: {0}\n{1}")]
    Header(#[from] headers::Error, Backtrace),
    #[error("IO error: {0}\n{1}")]
    File(#[from] std::io::Error, Backtrace),
    #[error("json error: {0}\n{1}")]
    Json(#[from] serde_json::Error, Backtrace),
    #[error("webserver error: {0}\n{1}")]
    Hyper(#[from] hyper::Error, Backtrace),
    #[error("webserver h3 error: {0}\n{1}")]
    H3(#[from] h3::Error, Backtrace),
    #[error("env var error: {0}\n{1}")]
    EnvVar(#[from] std::env::VarError, Backtrace),
    #[error("rustls error: {0}\n{1}")]
    Rustls(#[from] tokio_rustls::rustls::Error, Backtrace),
    #[error("poison error: {0}\n{1}")]
    Poison(#[from] std::sync::PoisonError<()>, Backtrace),
    #[error("join error: {0}\n{1}")]
    Join(#[from] tokio::task::JoinError, Backtrace),
    #[error("quic start error: {0}\n{1}")]
    S2nStart(#[from] s2n_quic::provider::StartError, Backtrace),
    #[error("configuration error: {0}")]
    Configuration(#[from] ConfigError),
    // #[cfg(feature = "perfect-group-allocation-telemetry")]
    //#[error("trace error: {0}")]
    //Trace(#[from] TraceError),
    #[error("database error: {0}\n{1}")]
    Database(#[from] DatabaseError, Backtrace),
    #[error("wrong csrf token")]
    WrongCsrfToken,
    #[error("no accept remaining")]
    NoAcceptRemaining,
    #[error(
        "The request session is still held onto. Maybe you keep it alive inside a streaming \
         response?"
    )]
    SessionStillHeld,
    #[error("openid connect error: {0}\n{1}")]
    OpenIdConnect(#[from] OpenIdConnectError, Backtrace),
    #[error(
        "Höchstwahrscheinlich ist deine Anmeldesession abgelaufen und du musst es erneut \
         versuchen. Wenn dies wieder auftritt, melde das Problem bitte an einen \
         Serveradministrator."
    )]
    OpenIdTokenNotFound,
    #[error("Der Serveradministrator hat OpenID nicht konfiguriert.")]
    OpenIdNotConfigured,
}

impl Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl From<Infallible> for AppError {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

impl From<diesel_async::pooled_connection::deadpool::PoolError> for AppError {
    fn from(value: diesel_async::pooled_connection::deadpool::PoolError) -> Self {
        Self::from(DatabaseError::from(value))
    }
}

impl AppError {
    pub fn build_error_template(
        self,
        session: &Session<'_, String>,
    ) -> Response<impl http_body::Body<Data = Bytes, Error = Infallible> + Send> {
        let result = async gen move {
            let template = yieldfi!(error());
            let template = yieldfi!(template.next());
            let template = yieldfi!(template.next());
            let template = yieldfv!(template.page_title("Internal Server Error"));
            let template = yieldfi!(template.next());
            let template = yieldfv!(
                template.indexcss_version_unsafe(Unsafe::unsafe_input(index_css!().1.to_string()))
            );
            let template = yieldfi!(template.next());
            let template = yieldfi!(template.next());
            let template = yieldfi!(template.next_email_false());
            let template = yieldfv!(template.csrf_token(session.get_csrf_token()));
            let template = yieldfi!(template.next());
            let template = yieldfi!(template.next());
            let template = yieldfi!(template.next());
            let template = yieldfv!(template.request_id("REQUESTID"));
            let template = yieldfi!(template.next());
            let template = yieldfv!(template.error(self.to_string()));
            let template = yieldfi!(template.next());
            yieldfi!(template.next());
        };
        let stream = AsyncIteratorStream(result);
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .typed_header(ContentType::html())
            .body(StreamBody::new(stream))
            .unwrap()
    }
}
