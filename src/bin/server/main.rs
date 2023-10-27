#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![feature(coroutines)]
#![feature(type_name_of_val)]

pub mod catch_panic;
pub mod csrf_protection;
mod entities;
mod error;
mod openid;
pub mod routes;
pub mod session;
use std::any::Any;
use std::borrow::Cow;
use std::convert::Infallible;
use std::fs::File;
use std::future::poll_fn;
use std::io::BufReader;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use anyhow::anyhow;
use axum::body::StreamBody;
use axum::error_handling::HandleErrorLayer;
use axum::extract::multipart::MultipartError;
use axum::extract::rejection::FormRejection;
use axum::extract::{BodyStream, FromRef, FromRequest, State};
use axum::headers::{self, Header};
use axum::http::{self, HeaderName, HeaderValue};
use axum::response::{Html, IntoResponse, IntoResponseParts, Redirect, Response};
use axum::routing::{get, post};
use axum::{async_trait, BoxError, Form, Router, TypedHeader};
use axum_extra::extract::cookie::{Cookie, Key};
use axum_extra::extract::PrivateCookieJar;
use axum_extra::response::Css;
use catch_panic::CatchPanicLayer;
use entities::prelude::*;
use entities::project_history;
use error::AppError;
use futures_async_stream::try_stream;
use futures_util::future::BoxFuture;
use futures_util::{StreamExt, TryStreamExt};
use handlebars::{
    Context as HandlebarsContext, Handlebars, Helper, HelperResult, Output, RenderContext,
};
use html_escape::encode_safe;
use http_body::Limited;
use hyper::server::accept::Accept;
use hyper::server::conn::{AddrIncoming, Http};
use hyper::{header, Method, Request, StatusCode};
use itertools::Itertools;
use lightningcss::bundler::{Bundler, FileProvider};
use lightningcss::stylesheet::{ParserOptions, PrinterOptions};
use lightningcss::targets::Targets;
use oauth2::basic::{BasicErrorResponseType, BasicTokenType};
use oauth2::{PkceCodeVerifier, StandardRevocableToken};
use openidconnect::core::{
    CoreAuthDisplay, CoreAuthPrompt, CoreAuthenticationFlow, CoreClient, CoreGenderClaim,
    CoreJsonWebKey, CoreJsonWebKeyType, CoreJsonWebKeyUse, CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm, CoreProviderMetadata,
};
use openidconnect::reqwest::async_http_client;
use openidconnect::{
    AccessTokenHash, AuthorizationCode, Client, ClientId, ClientSecret, EmptyAdditionalClaims,
    EmptyExtraTokenFields, IdTokenFields, IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge,
    RedirectUrl, RevocationErrorResponseType, Scope, StandardErrorResponse,
    StandardTokenIntrospectionResponse, StandardTokenResponse, TokenResponse,
};
use parcel_sourcemap::SourceMap;
use pin_project_lite::pin_project;
use rand::{thread_rng, Rng};
use rustls_pemfile::{certs, ec_private_keys, pkcs8_private_keys};
use sea_orm::{
    ActiveValue, ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, EntityTrait,
    RuntimeErr, Statement,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use session::Session;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
use tokio_rustls::TlsAcceptor;
use tokio_util::io::ReaderStream;
use tower::make::MakeService;
use tower::{Layer, Service, ServiceBuilder};
use tower_http::compression::CompressionLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::{
    RequestBodyTimeoutLayer, ResponseBodyTimeoutLayer, TimeoutBody, TimeoutLayer,
};
use tower_http::trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnResponse, TraceLayer};

const DB_NAME: &str = "postgres";

trait CsrfSafeExtractor {}

struct ExtractSession<E: CsrfSafeExtractor> {
    extractor: E,
    session: Arc<Mutex<Session>>,
}

#[async_trait]
impl<S, B, T: CsrfSafeExtractor> FromRequest<S, BodyWithSession<B>> for ExtractSession<T>
where
    B: Send + 'static,
    S: Send + Sync,
    T: FromRequest<S, BodyWithSession<B>>,
{
    type Rejection = T::Rejection;

    async fn from_request(
        req: Request<BodyWithSession<B>>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();
        let session = body.session.clone();
        let extractor = T::from_request(Request::from_parts(parts, body), state).await?;
        Ok(Self { extractor, session })
    }
}

pin_project! {
    pub struct BodyWithSession<B> {
        session: Arc<Mutex<Session>>,
        #[pin]
        body: B
    }
}

impl<B> http_body::Body for BodyWithSession<B>
where
    B: http_body::Body,
{
    type Data = B::Data;
    type Error = B::Error;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<Self::Data, Self::Error>>> {
        let this = self.project();

        this.body.poll_data(cx)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<Option<hyper::HeaderMap>, Self::Error>> {
        let this = self.project();

        this.body.poll_trailers(cx)
    }
}

type MyBody0 = hyper::Body;
type MyBody1 = Limited<MyBody0>;
type MyBody2 = TimeoutBody<MyBody1>;
type MyBody3 = BodyWithSession<MyBody2>;
type MyBody = MyBody3;

#[derive(Clone, FromRef)]
struct MyState {
    database: DatabaseConnection,
    handlebars: Handlebars<'static>,
}

// https://handlebarsjs.com/api-reference/
// https://handlebarsjs.com/api-reference/data-variables.html

#[derive(Serialize)]
pub struct CreateProject {
    csrf_token: String,
    title: Option<String>,
    title_error: Option<String>,
    description: Option<String>,
    description_error: Option<String>,
}

#[derive(Serialize)]
pub struct TemplateProject {
    title: String,
    description: String,
}

pub struct EmptyBody;

#[async_trait]
impl<S, B> FromRequest<S, B> for EmptyBody
where
    B: Send + 'static,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request(_req: Request<B>, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(EmptyBody)
    }
}

impl CsrfSafeExtractor for EmptyBody {}

#[derive(Deserialize)]
struct CreateProjectPayload {
    csrf_token: String,
    title: String,
    description: String,
}

impl CsrfToken for CreateProjectPayload {
    fn csrf_token(&self) -> String {
        self.csrf_token.clone()
    }
}

trait CsrfToken {
    fn csrf_token(&self) -> String;
}

#[derive(Deserialize)]
struct CsrfSafeForm<T: CsrfToken> {
    value: T,
}

#[async_trait]
impl<T, S, B> FromRequest<S, BodyWithSession<B>> for CsrfSafeForm<T>
where
    T: DeserializeOwned + CsrfToken,
    B: http_body::Body + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(
        req: Request<BodyWithSession<B>>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();

        let mut session = body.session.lock().await;
        let expected_csrf_token = session.session_id();
        drop(session);

        let not_get_or_head = !(parts.method == Method::GET || parts.method == Method::HEAD);

        let extractor = Form::<T>::from_request(Request::from_parts(parts, body), state)
            .await
            .map_err(AppError::from)?;

        if not_get_or_head {
            let actual_csrf_token = extractor.0.csrf_token();

            if expected_csrf_token != actual_csrf_token {
                return Err(AppError::WrongCsrfToken);
            }
        }
        Ok(Self { value: extractor.0 })
    }
}

impl<T: CsrfToken> CsrfSafeExtractor for CsrfSafeForm<T> {}

// TODO rtt0
fn rustls_server_config(key: impl AsRef<Path>, cert: impl AsRef<Path>) -> Arc<ServerConfig> {
    let mut key_reader = BufReader::new(File::open(key).unwrap());
    let mut cert_reader = BufReader::new(File::open(cert).unwrap());

    let key = PrivateKey(pkcs8_private_keys(&mut key_reader).unwrap().remove(0));
    let certs = certs(&mut cert_reader)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();

    let mut config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .expect("bad certificate/key");

    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Arc::new(config)
}

// maybe not per request csrf but per form a different csrf token that is only valid for the form as defense in depth.

fn csrf_helper(
    h: &Helper<'_, '_>,
    _hb: &Handlebars<'_>,
    _c: &HandlebarsContext,
    _rc: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    // get parameter from helper or throw an error
    let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    //c.data()
    //rc.context()

    // is one of the always the top level context from which we could get the csrf token? Also we can create
    // helpers as a struct and then store data in them so maybe register the helper per http handler and configure
    // the user there?

    out.write(param.to_uppercase().as_ref())?;
    Ok(())
}

// https://github.com/sunng87/handlebars-rust/tree/master/src/helpers
// https://github.com/sunng87/handlebars-rust/blob/master/src/helpers/helper_with.rs
// https://github.com/sunng87/handlebars-rust/blob/master/src/helpers/helper_lookup.rs

struct XRequestId(String);

static X_REQUEST_ID_HEADER_NAME: HeaderName = http::header::HeaderName::from_static("x-request-id");

impl Header for XRequestId {
    fn name() -> &'static HeaderName {
        &X_REQUEST_ID_HEADER_NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values
            .exactly_one()
            .map_err(|e| headers::Error::invalid())?;
        let value = value.to_str().map_err(|e| headers::Error::invalid())?;
        Ok(XRequestId(value.to_string()))
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        let value = HeaderValue::from_str(&self.0).unwrap();

        values.extend(std::iter::once(value));
    }
}

async fn handle_error_test(
    TypedHeader(request_id): TypedHeader<XRequestId>,
    err: Box<dyn std::error::Error + Sync + Send + 'static>,
) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!(
            "Unhandled internal error for request {} {:?}",
            request_id.0, err
        ),
    )
}

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL env must be set");

    let db = Database::connect(&database_url).await?;

    let db = match db.get_database_backend() {
        DbBackend::MySql => {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE IF NOT EXISTS `{DB_NAME}`;"),
            ))
            .await?;

            let url = format!("{database_url}/{DB_NAME}");
            Database::connect(&url).await?
        }
        DbBackend::Postgres => {
            let err_already_exists = db
                .execute(Statement::from_string(
                    db.get_database_backend(),
                    format!("CREATE DATABASE \"{DB_NAME}\";"),
                ))
                .await;

            match err_already_exists {
                Err(DbErr::Exec(RuntimeErr::SqlxError(sqlx::Error::Database(err))))
                    if err.code() == Some(Cow::Borrowed("42P04")) =>
                {
                    // database already exists error
                }
                Err(err) => panic!("{}", err),
                Ok(_) => {}
            }

            let url = format!("{database_url}/{DB_NAME}");
            Database::connect(&url).await?
        }
        DbBackend::Sqlite => db,
    };

    // https://github.com/tokio-rs/axum/discussions/236

    // https://github.com/tokio-rs/axum/blob/main/examples/low-level-rustls/src/main.rs

    let rustls_config = rustls_server_config(
        ".lego/certificates/h3.selfmade4u.de.key",
        ".lego/certificates/h3.selfmade4u.de.crt",
    );

    let acceptor = TlsAcceptor::from(rustls_config);

    let listener = TcpListener::bind("127.0.0.1:8443").await.unwrap();
    let mut listener = AddrIncoming::from_listener(listener).unwrap();

    let http = Http::new();

    let protocol = Arc::new(http);

    let service = ServeDir::new("frontend");

    let mut handlebars = Handlebars::new();
    handlebars.set_dev_mode(true);
    handlebars.set_strict_mode(true);

    handlebars
        .register_templates_directory(".hbs", "./templates/")
        .unwrap();

    //  RUST_LOG=tower_http::trace=TRACE cargo run --bin server
    let app: Router<MyState, MyBody> = Router::new()
        .route("/", get(index))
        .route("/", post(create))
        .route("/index.css", get(indexcss))
        .route("/list", get(list))
        .route("/download", get(handler))
        .route("/openidconnect-login", post(openid_login))
        .route("/openidconnect-redirect", post(openid_redirect))
        .fallback_service(service);

    // layers are in reverse order
    let app: Router<MyState, MyBody2> = app.layer(SessionLayer {
        key: Key::generate(),
    });
    let app: Router<MyState, MyBody2> = app.layer(CompressionLayer::new());
    let app: Router<MyState, MyBody2> =
        app.layer(ResponseBodyTimeoutLayer::new(Duration::from_secs(10)));
    let app: Router<MyState, MyBody1> =
        app.layer(RequestBodyTimeoutLayer::new(Duration::from_secs(10))); // this timeout is between sends, so not the total timeout
    let app: Router<MyState, MyBody0> = app.layer(RequestBodyLimitLayer::new(100 * 1024 * 1024));
    let app: Router<MyState, MyBody0> = app.layer(TimeoutLayer::new(Duration::from_secs(5)));
    let app: Router<MyState, MyBody0> = app.layer(SetResponseHeaderLayer::overriding(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "base-uri 'none'; default-src 'none'; style-src 'self'; img-src 'self'; form-action \
             'self'; frame-ancestors 'none'; sandbox allow-forms allow-same-origin; \
             upgrade-insecure-requests; require-trusted-types-for 'script'; trusted-types a",
        ),
    ));
    // don't ask, thanks
    let app: Router<MyState, MyBody0> = app.layer(SetResponseHeaderLayer::overriding(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static(
            "accelerometer=(), ambient-light-sensor=(), attribution-reporting=(), autoplay=(), \
             battery=(), camera=(), display-capture=(), document-domain=(), encrypted-media=(), \
             execution-while-not-rendered=(), execution-while-out-of-viewport=(), fullscreen=(), \
             gamepad=(), gamepad=(), gyroscope=(), hid=(), identity-credentials-get=(), \
             idle-detection=(), local-fonts=(), magnetometer=(), microphone=(), midi=(), \
             otp-credentials=(), payment=(), picture-in-picture=(), \
             publickey-credentials-create=(), publickey-credentials-get=(), screen-wake-lock=(), \
             serial=(), speaker-selection=(), storage-access=(), usb=(), web-share=(), \
             window-management=(), xr-spatial-tracking=();",
        ),
    ));
    let app: Router<MyState, MyBody0> = app.layer(SetResponseHeaderLayer::overriding(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=63072000; preload"),
    ));
    // https://cheatsheetseries.owasp.org/cheatsheets/Content_Security_Policy_Cheat_Sheet.html
    // TODO FIXME sandbox is way too strict
    // https://csp-evaluator.withgoogle.com/
    // https://web.dev/articles/strict-csp
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Security-Policy
    // cat frontend/index.css | openssl dgst -sha256 -binary | openssl enc -base64
    let app: Router<MyState, MyBody0> = app.layer(SetResponseHeaderLayer::overriding(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    ));
    let app: Router<MyState, MyBody0> = app.layer(SetResponseHeaderLayer::overriding(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-store, must-revalidate"),
    ));
    let app: Router<(), MyBody0> = app.with_state(MyState {
        database: db,
        handlebars,
    });
    //let app: Router<(), MyBody0> = app.layer(PropagateRequestIdLayer::x_request_id());
    let app = app.layer(
        ServiceBuilder::new()
            .layer(HandleErrorLayer::new(handle_error_test))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::default().include_headers(true))
                    .on_response(DefaultOnResponse::default().include_headers(true)),
            )
            .layer(CatchPanicLayer),
    );
    let app: Router<(), MyBody0> = app.layer(SetRequestIdLayer::x_request_id(MakeRequestUuid));

    let mut app = app.into_make_service();

    loop {
        let stream = poll_fn(|cx| Pin::new(&mut listener).poll_accept(cx))
            .await
            .unwrap()
            .unwrap();

        let acceptor = acceptor.clone();

        let protocol = protocol.clone();

        let svc = MakeService::<_, Request<hyper::Body>>::make_service(&mut app, &stream);

        tokio::spawn(async move {
            if let Ok(stream) = acceptor.accept(stream).await {
                let _ = protocol.serve_connection(stream, svc.await.unwrap()).await;
            }
        });
    }
}
