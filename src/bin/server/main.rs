#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![feature(generators)]

mod entities;
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

use axum::body::StreamBody;
use axum::extract::multipart::MultipartError;
use axum::extract::{BodyStream, FromRef, FromRequest, FromRequestParts, Multipart, State};
use axum::http::request::Parts;
use axum::http::HeaderValue;
use axum::response::{Html, IntoResponse, IntoResponseParts, Redirect, Response};
use axum::routing::{get, post};
use axum::{
    async_trait, BoxError, Extension, RequestExt, RequestPartsExt, Router,
    ServiceExt as AxumServiceExt,
};
use axum_extra::extract::cookie::{Cookie, Key};
use axum_extra::extract::{CookieJar, SignedCookieJar};
use data_encoding::{BASE64, BASE64URL_NOPAD};
use entities::prelude::*;
use entities::project_history;
use futures_async_stream::try_stream;
use futures_util::future::BoxFuture;
use futures_util::{StreamExt, TryStreamExt};
use handlebars::{
    Context as HandlebarsContext, Handlebars, Helper, HelperResult, Output, RenderContext,
};
use html_escape::encode_safe;
use http_body::combinators::UnsyncBoxBody;
use http_body::{Body, Limited};
use hyper::server::accept::Accept;
use hyper::server::conn::{AddrIncoming, Http};
use hyper::{header, Request, StatusCode};
use pin_project_lite::pin_project;
use rand::{thread_rng, Rng};
use ring::hmac;
use rustls_pemfile::{certs, ec_private_keys};
use sea_orm::{
    ActiveValue, ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, EntityTrait,
    RuntimeErr, Statement,
};
use serde::Serialize;
use serde_json::json;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
use tokio_rustls::TlsAcceptor;
use tokio_util::io::ReaderStream;
use tower::make::MakeService;
use tower::{Layer, Service, ServiceExt as TowerServiceExt};
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::compression::CompressionLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::{
    RequestBodyTimeoutLayer, ResponseBodyTimeoutLayer, TimeoutBody, TimeoutLayer,
};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};

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
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
    type Response = S::Response;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        let (parts, body) = request.into_parts();
        let session = Arc::new(Mutex::new(Session::new(SignedCookieJar::from_headers(
            &parts.headers,
            self.key.clone(),
        ))));
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

#[derive(Clone)]
struct Session {
    signed_cookies: SignedCookieJar,
}

impl Session {
    pub fn new(signed_cookies: SignedCookieJar) -> Self {
        Self { signed_cookies }
    }

    pub fn session_id(&mut self) -> String {
        const COOKIE_NAME: &str = "__Host-session_id";
        if self.signed_cookies.get(COOKIE_NAME).is_none() {
            let rand_string: String = thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(30)
                .map(char::from)
                .collect();

            let session_id = rand_string;
            let cookie = Cookie::build(COOKIE_NAME, session_id)
                .http_only(true)
                .same_site(axum_extra::extract::cookie::SameSite::Lax)
                .secure(true)
                .finish();
            self.signed_cookies = self.signed_cookies.clone().add(cookie);
        }
        self.signed_cookies
            .get(COOKIE_NAME)
            .map(|c| c.value().to_string())
            .unwrap()
    }

    pub fn csrf_token(&mut self) -> String {
        const COOKIE_NAME: &str = " __Host-csrf_token";
        if self.signed_cookies.get(COOKIE_NAME).is_none() {
            let rand_string: String = thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(30)
                .map(char::from)
                .collect();

            let csrf_token = self.session_id() + ":" + &rand_string;

            let cookie = Cookie::build(COOKIE_NAME, csrf_token)
                .http_only(true)
                .same_site(axum_extra::extract::cookie::SameSite::Lax)
                .secure(true)
                .finish();
            self.signed_cookies = self.signed_cookies.clone().add(cookie);
        }
        // TODO FIXME unwrap
        self.signed_cookies
            .get(COOKIE_NAME)
            .map(|c| c.value().to_string())
            .unwrap()
            .split_once(":")
            .unwrap()
            .1
            .to_string()
    }
}

// fromrequestparts: TODO we could extract it from parts.extensions but then its not 100% type safe

struct ExtractSession<E> {
    extractor: E,
    session: Arc<Mutex<Session>>,
}

#[async_trait]
impl<S, B, T> FromRequest<S, BodyWithSession<B>> for ExtractSession<T>
where
    B: Send + 'static,
    S: Send + Sync,
    T: FromRequest<S, B>,
{
    type Rejection = T::Rejection;

    async fn from_request(
        req: Request<BodyWithSession<B>>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();
        let extractor = T::from_request(Request::from_parts(parts, body.body), state).await?;
        Ok(ExtractSession {
            extractor,
            session: body.session,
        })
    }
}

impl IntoResponseParts for Session {
    type Error = Infallible;

    fn into_response_parts(
        self,
        res: axum::response::ResponseParts,
    ) -> Result<axum::response::ResponseParts, Self::Error> {
        self.signed_cookies.into_response_parts(res)
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
// Make our own error that wraps `anyhow::Error`.
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        println!("{}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl From<MultipartError> for AppError {
    fn from(err: MultipartError) -> Self {
        Self(err.into())
    }
}
impl From<axum::Error> for AppError {
    fn from(err: axum::Error) -> Self {
        Self(err.into())
    }
}
impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        Self(err.into())
    }
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

#[axum::debug_handler(body=MyBody, state=MyState)]
async fn index(
    handlebars: State<Handlebars<'static>>,
    ExtractSession { extractor, session }: ExtractSession<String>, // TODO FIXME empty extractor
) -> impl IntoResponse {
    let result = handlebars
        .render(
            "create-project",
            &CreateProject {
                csrf_token: session.lock().await.csrf_token(),
                title: None,
                title_error: None,
                description: None,
                description_error: None,
            },
        )
        .unwrap_or_else(|e| e.to_string());
    Html(result)
}

//#[axum::debug_handler(body=MyBody, state=MyState)]
async fn create(
    State(db): State<DatabaseConnection>,
    State(handlebars): State<Handlebars<'static>>,
    ExtractSession {
        extractor: mut multipart,
        session,
    }: ExtractSession<Multipart>,
) -> Result<impl IntoResponse, AppError> {
    println!("{}", session.lock().await.session_id());

    let mut title = None;
    let mut description = None;
    while let Some(field) = multipart.next_field().await? {
        match field.name().unwrap() {
            "title" => assert!(title.replace(field.text().await?).is_none()),
            "description" => assert!(description.replace(field.text().await?).is_none()),
            "CSRFToken" => {}
            v => panic!("unexpected field {v}"),
        }
    }
    let title = title.unwrap();
    let description = description.unwrap();

    let mut title_error = None;
    let mut description_error = None;

    if title.is_empty() {
        title_error = Some("title must not be empty".to_string());
    }

    if description.is_empty() {
        description_error = Some("description must not be empty".to_string());
    }

    if title_error.is_some() || description_error.is_some() {
        let result = handlebars
            .render(
                "create-project",
                &CreateProject {
                    csrf_token: session.lock().await.csrf_token(),
                    title: Some(title),
                    title_error,
                    description: Some(description),
                    description_error,
                },
            )
            .unwrap_or_else(|e| e.to_string());
        return Ok(Html(result).into_response());
    }

    let project = project_history::ActiveModel {
        id: ActiveValue::Set(1),
        title: ActiveValue::Set(title),
        description: ActiveValue::Set(description),
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
    while let Some(chunk) = stream.try_next().await? {
        println!("{chunk:?}");
    }
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

// TODO rtt0
fn rustls_server_config(key: impl AsRef<Path>, cert: impl AsRef<Path>) -> Arc<ServerConfig> {
    let mut key_reader = BufReader::new(File::open(key).unwrap());
    let mut cert_reader = BufReader::new(File::open(cert).unwrap());

    let key = PrivateKey(ec_private_keys(&mut key_reader).unwrap().remove(0));
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
        .route("/list", get(list))
        .route("/download", get(handler))
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
    let app: Router<MyState, MyBody0> = app.layer(CatchPanicLayer::new());
    let app: Router<MyState, MyBody0> = app.layer(TimeoutLayer::new(Duration::from_secs(5)));
    let app: Router<MyState, MyBody0> = app.layer(SetResponseHeaderLayer::overriding(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "base-uri 'none'; default-src 'none'; style-src 'self'; img-src 'self'; form-action \
             'self'; frame-ancestors 'none'; sandbox allow-forms; upgrade-insecure-requests; \
             require-trusted-types-for 'script'; trusted-types a",
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
    let app: Router<(), MyBody0> = app.layer(PropagateRequestIdLayer::x_request_id());
    let app: Router<(), MyBody0> = app.layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            .on_response(DefaultOnResponse::new().include_headers(true)),
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
