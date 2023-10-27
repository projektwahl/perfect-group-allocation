#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![feature(coroutines)]
#![feature(type_name_of_val)]

pub mod catch_panic;
mod entities;
mod error;
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

#[derive(Clone)]
struct SessionLayer {
    key: Key,
}

impl<S> Layer<S> for SessionLayer {
    type Service = SessionMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SessionMiddleware {
            inner,
            key: self.key.clone(),
        }
    }
}

#[derive(Clone)]
struct SessionMiddleware<S> {
    inner: S,
    key: Key,
}

impl<S, ReqBody> Service<Request<ReqBody>> for SessionMiddleware<S>
where
    S: Service<Request<BodyWithSession<ReqBody>>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
    type Response = S::Response;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        let (parts, body) = request.into_parts();
        let session = Session::new(PrivateCookieJar::from_headers(
            &parts.headers,
            self.key.clone(),
        ));
        let session = Arc::new(Mutex::new(session));
        let future = self.inner.call(Request::from_parts(
            parts,
            BodyWithSession {
                session: session.clone(),
                body,
            },
        ));
        Box::pin(async move {
            let response: Response = future.await?;
            let cookies = Arc::into_inner(session).unwrap().into_inner();
            Ok((cookies, response).into_response())
        })
    }
}

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

#[axum::debug_handler(body=MyBody, state=MyState)]
async fn index(
    handlebars: State<Handlebars<'static>>,
    ExtractSession {
        extractor: _,
        session,
    }: ExtractSession<EmptyBody>,
) -> impl IntoResponse {
    let result = handlebars
        .render(
            "create-project",
            &CreateProject {
                csrf_token: session.lock().await.session_id(),
                title: None,
                title_error: None,
                description: None,
                description_error: None,
            },
        )
        .unwrap_or_else(|e| e.to_string());
    Html(result)
}

#[axum::debug_handler(body=MyBody, state=MyState)]
async fn indexcss() -> impl IntoResponse {
    // @import would produce a flash of unstyled content and also is less efficient
    let fs = FileProvider::new();
    let mut bundler = Bundler::new(&fs, None, ParserOptions::default());
    let stylesheet = bundler.bundle(Path::new("frontend/index.css")).unwrap();
    let mut source_map = SourceMap::new(".");
    Css(stylesheet
        .to_css(PrinterOptions {
            minify: true,
            source_map: Some(&mut source_map),
            project_root: None,
            targets: Targets::default(),
            analyze_dependencies: None,
            pseudo_classes: None,
        })
        .unwrap()
        .code)
}

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

#[axum::debug_handler(body=MyBody, state=MyState)]
async fn create(
    State(db): State<DatabaseConnection>,
    State(handlebars): State<Handlebars<'static>>,
    ExtractSession {
        extractor: form,
        session,
    }: ExtractSession<CsrfSafeForm<CreateProjectPayload>>,
) -> Result<impl IntoResponse, AppError> {
    let mut title_error = None;
    let mut description_error = None;

    if form.value.title.is_empty() {
        title_error = Some("title must not be empty".to_string());
    }

    if form.value.description.is_empty() {
        description_error = Some("description must not be empty".to_string());
    }

    if title_error.is_some() || description_error.is_some() {
        let result = handlebars
            .render(
                "create-project",
                &CreateProject {
                    csrf_token: session.lock().await.session_id(),
                    title: Some(form.value.title.clone()),
                    title_error,
                    description: Some(form.value.description.clone()),
                    description_error,
                },
            )
            .unwrap_or_else(|e| e.to_string());
        return Ok(Html(result).into_response());
    }

    let project = project_history::ActiveModel {
        id: ActiveValue::Set(1),
        title: ActiveValue::Set(form.value.title.clone()),
        description: ActiveValue::Set(form.value.description.clone()),
        ..Default::default()
    };
    let _ = ProjectHistory::insert(project).exec(&db).await?;

    Ok(Redirect::to("/list").into_response())
}

#[try_stream(ok = String, error = DbErr)]
async fn list_internal(db: DatabaseConnection, handlebars: Handlebars<'static>) {
    let stream = ProjectHistory::find().stream(&db).await.unwrap();
    yield handlebars
        .render("main_pre", &json!({"page_title": "Projects"}))
        .unwrap_or_else(|e| e.to_string());
    #[for_await]
    for x in stream {
        let x = x?;
        let result = handlebars
            .render(
                "project",
                &TemplateProject {
                    title: x.title,
                    description: x.description,
                },
            )
            .unwrap_or_else(|e| e.to_string());
        yield result;
    }
    yield handlebars
        .render("main_post", &json!({}))
        .unwrap_or_else(|e| e.to_string());
}

#[axum::debug_handler(body=MyBody, state=MyState)]
async fn list(
    State(db): State<DatabaseConnection>,
    State(handlebars): State<Handlebars<'static>>,
) -> impl IntoResponse {
    let stream = list_internal(db, handlebars).map(|elem| match elem {
        Err(v) => Ok(format!("<h1>Error {}</h1>", encode_safe(&v.to_string()))),
        o => o,
    });
    (
        [(header::CONTENT_TYPE, "text/html")],
        StreamBody::new(stream),
    )
}

#[axum::debug_handler(body=MyBody, state=MyState)]
async fn handler(mut stream: BodyStream) -> Result<impl IntoResponse, AppError> {
    while let Some(_chunk) = stream.try_next().await? {}
    let file = tokio::fs::File::open("/var/cache/pacman/pkg/firefox-118.0.2-1-x86_64.pkg.tar.zst")
        .await
        .unwrap();
    let stream = ReaderStream::new(file);
    let body = hyper::Body::wrap_stream(stream);

    let headers = [
        (header::CONTENT_TYPE, "application/octet-stream"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"firefox-118.0.2-1-x86_64.pkg.tar.zst\"",
        ),
    ];

    Ok((headers, hyper::Response::new(body)))
}

async fn get_openid_client() -> Result<
    Client<
        EmptyAdditionalClaims,
        CoreAuthDisplay,
        CoreGenderClaim,
        CoreJweContentEncryptionAlgorithm,
        CoreJwsSigningAlgorithm,
        CoreJsonWebKeyType,
        CoreJsonWebKeyUse,
        CoreJsonWebKey,
        CoreAuthPrompt,
        StandardErrorResponse<BasicErrorResponseType>,
        StandardTokenResponse<
            IdTokenFields<
                EmptyAdditionalClaims,
                EmptyExtraTokenFields,
                CoreGenderClaim,
                CoreJweContentEncryptionAlgorithm,
                CoreJwsSigningAlgorithm,
                CoreJsonWebKeyType,
            >,
            BasicTokenType,
        >,
        BasicTokenType,
        StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
        StandardRevocableToken,
        StandardErrorResponse<RevocationErrorResponseType>,
    >,
    AppError,
> {
    let provider_metadata = CoreProviderMetadata::discover_async(
        IssuerUrl::new("https://accounts.example.com".to_string())?,
        async_http_client,
    )
    .await?;

    // Create an OpenID Connect client by specifying the client ID, client secret, authorization URL
    // and token URL.
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new("client_id".to_string()),
        Some(ClientSecret::new("client_secret".to_string())),
    )
    // Set the URL the user will be redirected to after the authorization process.
    .set_redirect_uri(RedirectUrl::new("http://redirect".to_string())?);
    Ok(client)
}

#[axum::debug_handler(body=MyBody, state=MyState)]
async fn openid_login(
    State(db): State<DatabaseConnection>,
    ExtractSession {
        extractor: form,
        session,
    }: ExtractSession<CsrfSafeForm<CreateProjectPayload>>,
) -> Result<impl IntoResponse, AppError> {
    let client = get_openid_client().await?;

    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, csrf_token, nonce) = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            openidconnect::CsrfToken::new_random,
            Nonce::new_random,
        )
        // Set the desired scopes.
        .add_scope(Scope::new("read".to_string()))
        .add_scope(Scope::new("write".to_string()))
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();

    let mut session = session.lock().await;

    session.set_openid_pkce_verifier(pkce_verifier);
    session.set_openid_nonce(nonce);

    // This is the URL you should redirect the user to, in order to trigger the authorization
    // process.
    println!("Browse to: {}", auth_url);

    Ok(Redirect::to("/list").into_response())
}

#[axum::debug_handler(body=MyBody, state=MyState)]
async fn openid_redirect(
    State(db): State<DatabaseConnection>,
    ExtractSession {
        extractor: form,
        session,
    }: ExtractSession<CsrfSafeForm<CreateProjectPayload>>,
) -> Result<impl IntoResponse, AppError> {
    let client = get_openid_client().await?;
    // Once the user has been redirected to the redirect URL, you'll have access to the
    // authorization code. For security reasons, your code should verify that the `state`
    // parameter returned by the server matches `csrf_state`.

    let session = session.lock().await;

    let pkce_verifier = session.openid_pkce_verifier();
    let nonce = session.openid_nonce();

    // Now you can exchange it for an access token and ID token.
    let token_response = client
        .exchange_code(AuthorizationCode::new(
            "some authorization code".to_string(),
        ))
        // Set the PKCE code verifier.
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await?;

    // Extract the ID token claims after verifying its authenticity and nonce.
    let id_token = token_response
        .id_token()
        .ok_or_else(|| anyhow!("Server did not return an ID token"))?;
    let claims = id_token.claims(&client.id_token_verifier(), &nonce)?;

    // Verify the access token hash to ensure that the access token hasn't been substituted for
    // another user's.
    if let Some(expected_access_token_hash) = claims.access_token_hash() {
        let actual_access_token_hash =
            AccessTokenHash::from_token(token_response.access_token(), &id_token.signing_alg()?)?;
        if actual_access_token_hash != *expected_access_token_hash {
            Err(anyhow!("Invalid access token"))?;
        }
    }

    // The authenticated user's identity is now available. See the IdTokenClaims struct for a
    // complete listing of the available claims.
    println!(
        "User {} with e-mail address {} has authenticated successfully",
        claims.subject().as_str(),
        claims
            .email()
            .map(|email| email.as_str())
            .unwrap_or("<not provided>"),
    );
    Ok(Redirect::to("/list").into_response())
}

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
