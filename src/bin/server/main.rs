#![feature(generators)]

mod entities;
use std::fs::File;
use std::future::poll_fn;
use std::io::BufReader;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use axum::body::StreamBody;
use axum::extract::multipart::MultipartError;
use axum::extract::BodyStream;
use axum::extract::Multipart;
use axum::extract::State;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::routing::post;
use axum::Router;
use entities::project_history;
use entities::{prelude::*, *};
use futures_async_stream::stream;
use futures_async_stream::try_stream;
use futures_util::stream;
use futures_util::FutureExt;
use futures_util::Stream;
use futures_util::StreamExt;
use futures_util::TryStreamExt;
use http_body::Limited;
use hyper::header;
use hyper::server::accept::Accept;
use hyper::server::conn::AddrIncoming;
use hyper::server::conn::Http;
use hyper::Request;
use hyper::StatusCode;
use rustls_pemfile::certs;
use rustls_pemfile::ec_private_keys;
use sea_orm::ActiveValue;
use sea_orm::ConnectionTrait;
use sea_orm::Database;
use sea_orm::DatabaseConnection;
use sea_orm::DbBackend;
use sea_orm::DbErr;
use sea_orm::Statement;
use sea_orm::*;
use sea_orm_migration::MigratorTrait;
use sea_orm_migration::SchemaManager;
use tokio::net::TcpListener;
use tokio_rustls::rustls::Certificate;
use tokio_rustls::rustls::PrivateKey;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
use tokio_util::io::ReaderStream;
use tower::make::MakeService;
use tower::ServiceBuilder;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::compression::CompressionLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::MakeRequestUuid;
use tower_http::services::ServeDir;
use tower_http::timeout::RequestBodyTimeoutLayer;
use tower_http::timeout::ResponseBodyTimeoutLayer;
use tower_http::timeout::TimeoutBody;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::DefaultMakeSpan;
use tower_http::trace::DefaultOnResponse;
use tower_http::trace::TraceLayer;
use tower_http::ServiceBuilderExt;

const DATABASE_URL: &str = "sqlite:./sqlite.db?mode=rwc";
const DB_NAME: &str = "pga";

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

#[axum::debug_handler(body=MyBody, state=MyState)]
async fn index() -> impl IntoResponse {
    Html(include_str!("../../../frontend/form.html"))
}

#[axum::debug_handler(body=MyBody, state=MyState)]
async fn create(
    State(db): State<MyState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut title = None;
    let mut description = None;
    while let Some(mut field) = multipart.next_field().await? {
        match field.name().unwrap() {
            "title" => assert!(title.replace(field.text().await?).is_none()),
            "description" => assert!(description.replace(field.text().await?).is_none()),
            _ => panic!(),
        }
    }
    let title = title.unwrap();
    let description = description.unwrap();

    let project = project_history::ActiveModel {
        id: ActiveValue::Set(1),
        title: ActiveValue::Set(title),
        description: ActiveValue::Set(description),
        ..Default::default()
    };
    let _ = ProjectHistory::insert(project).exec(&db).await?;

    Ok(())
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
            x.title, x.description
        );
    }
    yield "THIS IS THE END".to_string();
}

#[axum::debug_handler(body=MyBody, state=MyState)]
async fn list(State(db): State<MyState>) -> impl IntoResponse {
    let stream = list_internal(db);
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
    let file =
        match tokio::fs::File::open("/var/cache/pacman/pkg/firefox-118.0.2-1-x86_64.pkg.tar.zst")
            .await
        {
            Ok(file) => file,
            Err(err) => panic!(),
        };
    // convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file);
    // convert the `Stream` into an `axum::body::HttpBody`
    let body = StreamBody::new(stream);

    let headers = [
        (header::CONTENT_TYPE, "application/octet-stream"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"firefox-118.0.2-1-x86_64.pkg.tar.zst\"",
        ),
    ];

    Ok((headers, body))
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

    config.alpn_protocols = vec![b"h2".to_vec()];

    Arc::new(config)
}

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    tracing_subscriber::fmt::init();

    let db = Database::connect(DATABASE_URL).await?;

    let db = match db.get_database_backend() {
        DbBackend::MySql => {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE IF NOT EXISTS `{}`;", DB_NAME),
            ))
            .await?;

            let url = format!("{}/{}", DATABASE_URL, DB_NAME);
            Database::connect(&url).await?
        }
        DbBackend::Postgres => {
            let err_already_exists = db
                .execute(Statement::from_string(
                    db.get_database_backend(),
                    format!("CREATE DATABASE \"{}\";", DB_NAME),
                ))
                .await;

            if let Err(err) = err_already_exists {
                println!("{err:?}");
            }

            let url = format!("{}/{}", DATABASE_URL, DB_NAME);
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

    let protocol = Arc::new(Http::new());

    let service = ServeDir::new("frontend");

    //  RUST_LOG=tower_http::trace=TRACE cargo run --bin server
    let mut app = Router::<MyState, MyBody>::new()
        .route("/", get(index))
        .route("/", post(create))
        .route("/list", get(list))
        .fallback_service(service)
        .with_state(db)
        .layer(
            ServiceBuilder::new()
                .set_x_request_id(MakeRequestUuid::default())
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().include_headers(true))
                        .on_response(DefaultOnResponse::new().include_headers(true)),
                )
                .propagate_x_request_id()
                .layer(TimeoutLayer::new(Duration::from_secs(5)))
                .layer(CatchPanicLayer::new())
                .layer(RequestBodyLimitLayer::new(100 * 1024 * 1024))
                .layer(RequestBodyTimeoutLayer::new(Duration::from_millis(100))) // this timeout is between sends, so not the total timeout
                .layer(ResponseBodyTimeoutLayer::new(Duration::from_secs(100)))
                .layer(CompressionLayer::new().quality(tower_http::CompressionLevel::Best)),
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
