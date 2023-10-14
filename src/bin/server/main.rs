#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![feature(generators)]

mod entities;
use std::borrow::Cow;
use std::fs::File;
use std::future::poll_fn;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use axum::body::StreamBody;
use axum::extract::multipart::MultipartError;
use axum::extract::{BodyStream, Multipart, State};
use axum::http::HeaderValue;
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::Router;
use entities::prelude::*;
use entities::project_history;
use futures_async_stream::try_stream;
use futures_util::{StreamExt, TryStreamExt};
use html_escape::encode_safe;
use http_body::Limited;
use hyper::server::accept::Accept;
use hyper::server::conn::{AddrIncoming, Http};
use hyper::{header, Request, StatusCode};
use rustls_pemfile::{certs, ec_private_keys};
use sea_orm::{
    ActiveValue, ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, EntityTrait,
    RuntimeErr, Statement,
};
use tokio::net::TcpListener;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
use tokio_rustls::TlsAcceptor;
use tokio_util::io::ReaderStream;
use tower::make::MakeService;
use tower::ServiceBuilder;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::compression::CompressionLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::MakeRequestUuid;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::{
    RequestBodyTimeoutLayer, ResponseBodyTimeoutLayer, TimeoutBody, TimeoutLayer,
};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tower_http::{CompressionLevel, ServiceBuilderExt};

const DB_NAME: &str = "postgres";

type MyBody = TimeoutBody<Limited<hyper::Body>>;

type MyState = DatabaseConnection;

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

#[try_stream(ok = String, error = DbErr)]
async fn index_template(
    title: Option<String>,
    description: Option<String>,
    title_error: Option<String>,
    description_error: Option<String>,
) {
    yield format!(
        r#"<!doctype html>
    <html lang="en">
    
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <title>Empty page</title>
        <link rel="stylesheet" href="index.css" />
    </head>
    
    <body>
        <main>
            <h1 class="center">Create project</h1>
            <form method="post" enctype="multipart/form-data">
                <label for="title">Title:</label>
                <input{} id="title" name="title" type="text"{} />
                {}

                <label for="description">Description:</label>
                <input{} id="description" name="description" type="text"{} />
                {}

                <button type="submit">Create</button>
            </form>
        </main>
    </body>
    
    </html>"#,
        if title_error.is_some() {
            r#" class="error""#
        } else {
            ""
        },
        title
            .map(|title| format!(r#" value="{}""#, encode_safe(&title)))
            .unwrap_or_default(),
        title_error
            .map(|title_error| format!(r#"<div class="error">{}</div>"#, encode_safe(&title_error)))
            .unwrap_or_default(),
        if description_error.is_some() {
            r#" class="error""#
        } else {
            ""
        },
        description
            .map(|description| format!(r#" value="{}""#, encode_safe(&description)))
            .unwrap_or_default(),
        description_error
            .map(|description_error| format!(
                r#"<div class="error">{}</div>"#,
                encode_safe(&description_error)
            ))
            .unwrap_or_default(),
    );
}

#[axum::debug_handler(body=MyBody, state=MyState)]
async fn index() -> impl IntoResponse {
    let stream = index_template(None, None, None, None);

    (
        [(header::CONTENT_TYPE, "text/html")],
        StreamBody::new(stream),
    )
}

#[axum::debug_handler(body=MyBody, state=MyState)]
async fn create(
    State(db): State<MyState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut title = None;
    let mut description = None;
    while let Some(field) = multipart.next_field().await? {
        match field.name().unwrap() {
            "title" => assert!(title.replace(field.text().await?).is_none()),
            "description" => assert!(description.replace(field.text().await?).is_none()),
            _ => panic!(),
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
        let stream = index_template(
            Some(title),
            Some(description),
            title_error,
            description_error,
        );

        return Ok((
            [(header::CONTENT_TYPE, "text/html")],
            StreamBody::new(stream),
        )
            .into_response());
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
async fn list_internal(db: DatabaseConnection) {
    let stream = ProjectHistory::find().stream(&db).await.unwrap();
    yield "THIS IS A TEST".to_string();
    #[for_await]
    for x in stream {
        let x = x?;
        yield format!(
            // TODO FIXME XSS
            "title: {}<br />description: {}<br /><br />",
            encode_safe(&x.title),
            encode_safe(&x.description)
        );
    }
    yield "THIS IS THE END".to_string();
}

#[axum::debug_handler(body=MyBody, state=MyState)]
async fn list(State(db): State<MyState>) -> impl IntoResponse {
    let stream = list_internal(db).map(|elem| match elem {
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

    let mut http = Http::new();

    let protocol = Arc::new(http);

    let service = ServeDir::new("frontend");

    //  RUST_LOG=tower_http::trace=TRACE cargo run --bin server
    let mut app = Router::<MyState, MyBody>::new()
        .route("/", get(index))
        .route("/", post(create))
        .route("/list", get(list))
        .route("/download", get(handler))
        .fallback_service(service)
        .with_state(db)
        .layer(
            ServiceBuilder::new()
                .set_x_request_id(MakeRequestUuid)
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().include_headers(true))
                        .on_response(DefaultOnResponse::new().include_headers(true)),
                )
                .propagate_x_request_id()
                .layer(SetResponseHeaderLayer::overriding(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("private, max-age=604800, must-revalidate"),
                ))
                .layer(TimeoutLayer::new(Duration::from_secs(5)))
                .layer(CatchPanicLayer::new())
                .layer(RequestBodyLimitLayer::new(100 * 1024 * 1024))
                .layer(RequestBodyTimeoutLayer::new(Duration::from_secs(10))) // this timeout is between sends, so not the total timeout
                .layer(ResponseBodyTimeoutLayer::new(Duration::from_secs(10)))
                .layer(CompressionLayer::new()),
        )
        .into_make_service();

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
