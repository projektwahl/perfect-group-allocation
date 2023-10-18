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
use std::time::Duration;

use axum::body::StreamBody;
use axum::extract::multipart::MultipartError;
use axum::extract::{BodyStream, FromRef, FromRequestParts, Multipart, State};
use axum::http::request::Parts;
use axum::http::HeaderValue;
use axum::middleware::{FromFn, FromFnLayer, Next};
use axum::response::{Html, IntoResponse, IntoResponseParts, Redirect, Response};
use axum::routing::{get, post};
use axum::{async_trait, BoxError, Extension, RequestPartsExt, Router};
use axum_extra::extract::cookie::{Cookie, Key};
use axum_extra::extract::{CookieJar, PrivateCookieJar};
use entities::prelude::*;
use entities::project_history;
use futures_async_stream::try_stream;
use futures_util::{StreamExt, TryStreamExt};
use handlebars::{
    handlebars_helper, Context, Handlebars, Helper, HelperResult, Output, RenderContext,
};
use html_escape::encode_safe;
use http_body::{Body, Limited};
use hyper::server::accept::Accept;
use hyper::server::conn::{AddrIncoming, Http};
use hyper::{header, Request, StatusCode};
use pin_project_lite::pin_project;
use rustls_pemfile::{certs, ec_private_keys};
use sea_orm::{
    ActiveValue, ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, EntityTrait,
    RuntimeErr, Statement,
};
use serde::Serialize;
use serde_json::json;
use tokio::net::TcpListener;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
use tokio_rustls::TlsAcceptor;
use tokio_util::io::ReaderStream;
use tower::layer::util::{Identity, Stack};
use tower::make::MakeService;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::compression::CompressionLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::{MakeRequestUuid, SetRequestIdLayer};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::{
    RequestBodyTimeoutLayer, ResponseBodyTimeoutLayer, TimeoutBody, TimeoutLayer,
};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tower_http::ServiceBuilderExt;

const DB_NAME: &str = "postgres";

pin_project! {
    pub struct WithSession<B> {
        #[pin]
        body: B
    }
}

impl<B> Body for WithSession<B>
where
    B: Body,
    B::Error: Into<BoxError>,
{
    type Data = B::Data;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<Self::Data, Self::Error>>> {
        let mut this = self.project();

        this.body.poll_data(cx).map_err(Into::into)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<Option<hyper::HeaderMap>, Self::Error>> {
        let mut this = self.project();

        this.body.poll_trailers(cx).map_err(Into::into)
    }
}

type MyBody0 = hyper::Body;
type MyBody1 = Limited<hyper::Body>;
type MyBody2 = TimeoutBody<Limited<hyper::Body>>;
type MyBody3 = WithSession<TimeoutBody<Limited<hyper::Body>>>;
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
async fn index(handlebars: State<Handlebars<'static>>) -> impl IntoResponse {
    let result = handlebars
        .render(
            "create-project",
            &CreateProject {
                csrf_token: "token".to_string(),
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
async fn create(
    State(db): State<DatabaseConnection>,
    State(handlebars): State<Handlebars<'static>>,
    session: Extension<Session>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
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
                    csrf_token: "test".to_string(),
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

    Ok((session, Redirect::to("/list")).into_response())
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
    hb: &Handlebars<'_>,
    c: &Context,
    rc: &mut RenderContext<'_, '_>,
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

#[derive(Clone)]
struct Session(PrivateCookieJar);

impl Session {
    pub fn new(mut cookies: PrivateCookieJar) -> Self {
        if cookies.get("session-id").is_none() {
            let session_id = "the-session-id";
            cookies = cookies.add(Cookie::new("session-id", session_id));
        }
        Self(cookies)
    }

    pub fn session_id(&self) -> Option<String> {
        self.0.get("session_id").map(|c| c.value().to_string())
    }
}

impl IntoResponseParts for Session {
    type Error = Infallible;

    fn into_response_parts(
        self,
        res: axum::response::ResponseParts,
    ) -> Result<axum::response::ResponseParts, Self::Error> {
        self.0.into_response_parts(res)
    }
}

async fn second_attempt_session<B>(
    cookies: PrivateCookieJar,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    request.extensions_mut().insert(Session::new(cookies));

    let response = next.run(request).await;

    Ok((
        response.extensions().get::<Session>().unwrap().clone(),
        response,
    )
        .into_response())
}

async fn third_attempt_body<B>(request: Request<B>, next: Next<B>) -> Response {
    //let request = request.map(|b| WithSession { body: b });
    let response = next.run(request).await;
    response
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

    let service_builder: ServiceBuilder<Identity> = ServiceBuilder::new();

    let service_builder: ServiceBuilder<Stack<SetRequestIdLayer<MakeRequestUuid>, Identity>> =
        service_builder.set_x_request_id(MakeRequestUuid);

    service_builder
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_response(DefaultOnResponse::new().include_headers(true)),
        )
        .propagate_x_request_id()
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-cache, no-store, must-revalidate"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=63072000; preload"),
        ))
        // https://cheatsheetseries.owasp.org/cheatsheets/Content_Security_Policy_Cheat_Sheet.html
        // TODO FIXME sandbox is way too strict
        // https://csp-evaluator.withgoogle.com/
        // https://web.dev/articles/strict-csp
        // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Security-Policy
        // cat frontend/index.css | openssl dgst -sha256 -binary | openssl enc -base64
        .layer(SetResponseHeaderLayer::overriding(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "base-uri 'none'; default-src 'none'; style-src 'self'; img-src 'self'; \
                 form-action 'self'; frame-ancestors 'none'; sandbox allow-forms; \
                 upgrade-insecure-requests; require-trusted-types-for 'script'; trusted-types a",
            ),
        ))
        .layer(TimeoutLayer::new(Duration::from_secs(5)))
        .layer(CatchPanicLayer::new())
        .layer(RequestBodyLimitLayer::new(100 * 1024 * 1024));

    //  RUST_LOG=tower_http::trace=TRACE cargo run --bin server
    let app: Router<(), MyBody3> = Router::new()
        .route("/", get(index))
        .route("/", post(create))
        .route("/list", get(list))
        .route("/download", get(handler))
        .fallback_service(service)
        .with_state(MyState {
            database: db,
            handlebars,
        });

    let from_fn: FromFnLayer<_, _, ()> =
        axum::middleware::from_fn(third_attempt_body::<hyper::Body>);
    // layers are in reverse order
    let app: Router<(), MyBody2> = app.layer(from_fn);
    let app: Router<(), MyBody3> = app.layer(CompressionLayer::new());
    let app: Router<(), MyBody3> =
        app.layer(ResponseBodyTimeoutLayer::new(Duration::from_secs(10)));
    let app: Router<(), MyBody2> = app.layer(RequestBodyTimeoutLayer::new(Duration::from_secs(10))); // this timeout is between sends, so not the total timeout

    let app = app.into_make_service();

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
