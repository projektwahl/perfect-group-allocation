use std::borrow::Cow;
use std::convert::Infallible;
use std::fmt::{Debug, Display};
use std::pin::pin;

use crate::components::main::main;

use crate::session::{ResponseSessionExt, Session};
use crate::ResponseTypedHeaderExt as _;
use async_zero_cost_templating::{html, TemplateToStream};
use bytes::Bytes;
use futures_util::StreamExt;
use headers::ContentType;
use http::{Response, StatusCode};

use perfect_group_allocation_config::{Config, ConfigError};
use perfect_group_allocation_database::DatabaseError;
use perfect_group_allocation_openidconnect::error::OpenIdConnectError;

#[derive(thiserror::Error)]
pub enum AppError {
    #[error("header error: {0}")]
    Header(#[from] headers::Error),
    #[error("IO error: {0}")]
    File(#[from] std::io::Error),
    #[error("Tls certificate failed to load {0}")]
    TlsCertificate(std::io::Error),
    #[error("Tls key failed to load {0}")]
    TlsKey(std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("webserver error: {0}")]
    Hyper(#[from] hyper::Error),
    //#[error("webserver h3 error: {0}")]
    //H3(#[from] h3::Error),
    #[error("env var error: {0}")]
    EnvVar(#[from] std::env::VarError),
    #[error("rustls error: {0}")]
    Rustls(#[from] tokio_rustls::rustls::Error),
    #[error("poison error: {0}")]
    Poison(#[from] std::sync::PoisonError<()>),
    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    //#[error("quic start error: {0}")]
    //S2nStart(#[from] s2n_quic::provider::StartError),
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
    #[must_use]
    pub async fn build_error_template(
        self,
        session: Session<Option<String>, ()>,
        config: Config,
    ) -> Response<impl http_body::Body<Data = Bytes, Error = Infallible> + Send + 'static> {
        let _csrf_token = session.csrf_token();
        let request_id = "REQUESTID";
        let error = self.to_string();
        let error = &error;
        let my_session = session.clone();

        let (tx_orig, rx) = tokio::sync::mpsc::channel(1);

        // TODO FIXME check that the error page can show you as logged in

        let tx = tx_orig.clone();
        let future = async move {
            html! {
                <div>
                    <h1 class="center">"Internal Server Error"</h1>

                    "Es ist ein interner Fehler aufgetreten! Bitte melde diesen an die Serveradministratoren. Ihnen können folgende
                    Informationen helfen:"<br>

                    "Request-ID: "(Cow::Borrowed(request_id))<br>
                    "Fehler: "(Cow::Borrowed(error))<br>
                </div>
            }
        };
        let future = main(
            tx_orig,
            "Internal Server Error".into(),
            &my_session,
            &config,
            future,
        );
        let stream = pin!(TemplateToStream::new(future, rx));
        // I think we should sent it at once with a content length when it is not too large
        let result = stream.collect::<String>().await;

        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .typed_header(ContentType::html())
            .with_session(session)
            .body(result)
            .unwrap()
    }
}
