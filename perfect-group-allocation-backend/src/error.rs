use std::backtrace::Backtrace;
use std::convert::Infallible;
use std::fmt::{Debug, Display};

use bytes::Bytes;
use headers::ContentType;
use http::{Response, StatusCode};
use http_body_util::StreamBody;
use perfect_group_allocation_config::ConfigError;
use perfect_group_allocation_database::DatabaseError;
use perfect_group_allocation_openidconnect::error::OpenIdConnectError;
use zero_cost_templating::async_iterator_extension::AsyncIteratorStream;
use zero_cost_templating::Unsafe;

use crate::routes::error;
use crate::routes::indexcss::INDEX_CSS_VERSION;
use crate::session::{ResponseSessionExt, Session};
use crate::{yieldfi, yieldfv, ResponseTypedHeaderExt as _};

#[derive(thiserror::Error)]
pub enum AppError {
    #[error("header error: {0}")]
    Header(#[from] headers::Error),
    #[error("IO error: {0}")]
    File(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("webserver error: {0}\n{1}")]
    Hyper(#[from] hyper::Error),
    #[error("webserver h3 error: {0}")]
    H3(#[from] h3::Error),
    #[error("env var error: {0}")]
    EnvVar(#[from] std::env::VarError),
    #[error("rustls error: {0}")]
    Rustls(#[from] tokio_rustls::rustls::Error),
    #[error("poison error: {0}")]
    Poison(#[from] std::sync::PoisonError<()>),
    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("quic start error: {0}")]
    S2nStart(#[from] s2n_quic::provider::StartError),
    #[error("configuration error: {0}")]
    Configuration(#[from] ConfigError),
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

impl From<diesel::result::Error> for AppError {
    fn from(value: diesel::result::Error) -> Self {
        Self::from(DatabaseError::from(value))
    }
}

impl AppError {
    pub fn build_error_template(
        self,
        session: Session<Option<String>, ()>,
    ) -> Response<impl http_body::Body<Data = Bytes, Error = Infallible> + Send + 'static> {
        let csrf_token = session.csrf_token();
        let result = async gen move {
            let template = yieldfi!(error());
            let template = yieldfi!(template.next());
            let template = yieldfi!(template.next());
            let template = yieldfv!(template.page_title("Internal Server Error"));
            let template = yieldfi!(template.next());
            let template = yieldfv!(
                template
                    .indexcss_version_unsafe(Unsafe::unsafe_input(INDEX_CSS_VERSION.to_string()))
            );
            let template = yieldfi!(template.next());
            let template = yieldfi!(template.next());
            let template = yieldfi!(template.next_email_false());
            let template = yieldfv!(template.csrf_token(csrf_token));
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
            .with_session(session)
            .body(StreamBody::new(stream))
            .unwrap()
    }
}
