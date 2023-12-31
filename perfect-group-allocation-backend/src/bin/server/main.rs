#![feature(gen_blocks)]
#![feature(lint_reasons)]
#![feature(let_chains)]
#![feature(hash_raw_entry)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::too_many_lines,
    reason = "not yet ready for that"
)]

extern crate alloc;

pub mod csrf_protection;
mod entities;
mod error;
mod openid;
pub mod router;
pub mod routes;
pub mod session;
pub mod telemetry;

use core::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{FromRef, FromRequest};
use axum::http::{self};
use axum::{async_trait, RequestExt, Router};
use axum_extra::extract::cookie::Key;
use error::{to_error_result, AppError};
use futures_util::pin_mut;
use http::{Request, StatusCode};
use hyper::body::Incoming;
use hyper::Method;
use hyper_util::rt::{TokioExecutor, TokioIo};
use routes::download::handler;
use routes::favicon::{favicon_ico, initialize_favicon_ico};
use routes::index::index;
use routes::indexcss::{indexcss, initialize_index_css};
use routes::openid_login::openid_login;
use routes::projects::create::create;
use sea_orm::{Database, DatabaseConnection};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use session::Session;
use telemetry::setup_telemetry;
use telemetry::trace_layer::MyTraceLayer;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::watch;
use tower::{service_fn, Service, ServiceExt as _};
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::services::ServeDir;
use tracing::{info, warn, Instrument as _};
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

use crate::openid::initialize_openid_client;
use crate::router::MyRouter;
use crate::routes::openid_redirect::openid_redirect;
use crate::routes::projects::list::list;
use crate::telemetry::tokio_metrics::tokio_runtime_metrics;

pub trait CsrfSafeExtractor {}

#[derive(Clone, FromRef)]
pub struct MyState {
    database: DatabaseConnection,
    key: Key,
}

// https://handlebarsjs.com/api-reference/
// https://handlebarsjs.com/api-reference/data-variables.html

#[derive(Serialize)]
pub struct CreateProject {
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

#[derive(Deserialize)]
pub struct CreateProjectPayload {
    csrf_token: String,
    title: String,
    description: String,
}

impl CsrfToken for CreateProjectPayload {
    fn csrf_token(&self) -> String {
        self.csrf_token.clone()
    }
}

pub trait CsrfToken {
    fn csrf_token(&self) -> String;
}

// TODO FIXME also provide session and request id through this so there is no duplicate extraction
#[derive(Deserialize)]
pub struct CsrfSafeForm<T: CsrfToken> {
    value: T,
}

#[async_trait]
impl<T> FromRequest<MyState> for CsrfSafeForm<T>
where
    T: DeserializeOwned + CsrfToken + Send,
{
    type Rejection = (Session, (StatusCode, axum::response::Response));

    async fn from_request(
        mut req: axum::extract::Request,
        state: &MyState,
    ) -> Result<Self, Self::Rejection> {
        let not_get_or_head = !(req.method() == Method::GET || req.method() == Method::HEAD);
        let session = match req
            .extract_parts_with_state::<Session, MyState>(state)
            .await
        {
            Ok(session) => session,
            Err(infallible) => match infallible {},
        };
        let expected_csrf_token = session.session().0;

        let result = async {
            #[expect(clippy::disallowed_types, reason = "this is the csrf safe wrapper")]
            let extractor = axum::Form::<T>::from_request(req, state).await?;

            if not_get_or_head {
                let actual_csrf_token = extractor.0.csrf_token();

                if expected_csrf_token != actual_csrf_token {
                    return Err(AppError::WrongCsrfToken);
                }
            }
            Ok(Self { value: extractor.0 })
        };
        match result.await {
            Ok(ok) => Ok(ok),
            Err(app_error) => Err(to_error_result(session, app_error).await),
        }
    }
}

impl<T: CsrfToken> CsrfSafeExtractor for CsrfSafeForm<T> {}

pub async fn get_database_connection() -> Result<DatabaseConnection, AppError> {
    let database_url = std::env::var("DATABASE_URL")?;

    let db = Database::connect(&database_url).await?;

    Ok(db)
}

//fn layers(_app: Router<MyState>, _db: DatabaseConnection) -> Router<()> {
// layers are in reverse order
//let app: Router<MyState, MyBody2> = app.layer(CompressionLayer::new()); // needs lots of compute power
//let app: Router<MyState, MyBody2> =
//    app.layer(ResponseBodyTimeoutLayer::new(Duration::from_secs(10)));
//let app: Router<MyState, MyBody1> =
//    app.layer(RequestBodyTimeoutLayer::new(Duration::from_secs(10))); // this timeout is between sends, so not the total timeout
//let app: Router<MyState, MyBody0> = app.layer(RequestBodyLimitLayer::new(100 * 1024 * 1024));
//let app: Router<MyState, MyBody0> = app.layer(TimeoutLayer::new(Duration::from_secs(5)));
/*let app: Router<MyState, MyBody0> = app.layer(SetResponseHeaderLayer::overriding(
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
*/
//}

// TODO https://github.com/tokio-rs/axum/tree/main/examples/auto-reload
// TODO https://github.com/tokio-rs/axum/tree/main/examples/consume-body-in-extractor-or-middleware for body length, download time etc metrics
// TODO https://github.com/tokio-rs/axum/blob/main/examples/error-handling/src/main.rs
// TODO https://github.com/tokio-rs/axum/blob/main/examples/global-404-handler/src/main.rs
// TODO https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs timeout handler
// TODO https://github.com/tokio-rs/axum/blob/main/examples/low-level-rustls/src/main.rs allow enabling rustls
// https://github.com/tokio-rs/axum/blob/main/examples/stream-to-file/src/main.rs
// https://github.com/tokio-rs/axum/blob/main/examples/tls-graceful-shutdown/src/main.rs graceful shutdown
// https://github.com/tokio-rs/axum/tree/main/examples/tls-rustls
// https://github.com/tokio-rs/axum/blob/main/examples/serve-with-hyper/src/main.rs
// https://github.com/tokio-rs/axum/blob/main/examples/listen-multiple-addrs/src/main.rs

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // avoid putting more code here as this is outside of all spans so doesn't get traced
    let _guard = setup_telemetry();

    program().await
}

#[tracing::instrument]
async fn program() -> Result<(), AppError> {
    tracing::Span::current().set_attribute(
        opentelemetry_semantic_conventions::trace::SERVER_ADDRESS,
        "localhost",
    );

    info!("starting up server...");

    tokio_runtime_metrics();

    initialize_favicon_ico().await;
    initialize_index_css();
    initialize_openid_client().await;

    let db = get_database_connection().await?;

    let service = ServeDir::new("frontend");

    let my_router = MyRouter::new()
        .route(&Method::GET, "/", index)
        .route(&Method::POST, "/", create)
        .route(&Method::GET, "/index.css", indexcss)
        .route(&Method::GET, "/favicon.ico", favicon_ico)
        .route(&Method::GET, "/list", list)
        .route(&Method::GET, "/download", handler)
        .route(&Method::POST, "/openidconnect-login", openid_login)
        .route(&Method::GET, "/openidconnect-redirect", openid_redirect);

    let app = my_router
        .finish()
        .route_service(
            "/test",
            service_fn(|req: http::Request<axum::body::Body>| async move {
                let body = axum::body::Body::from(format!("Hi from `{} /foo`", req.method()));
                let res = http::Response::new(body);
                Ok::<_, Infallible>(res)
            }),
        )
        .fallback_service(service);

    let app: Router<()> = app.with_state(MyState {
        database: db,
        key: Key::generate(),
    });
    let app = app.layer(CatchPanicLayer::new()).layer(MyTraceLayer);
    /*    let config = OpenSSLConfig::from_pem_file(
            ".lego/certificates/h3.selfmade4u.de.crt",
            ".lego/certificates/h3.selfmade4u.de.key",
        )
        .unwrap();
    */
    /*
        let config = RustlsConfig::from_pem_file(
            ".lego/certificates/h3.selfmade4u.de.crt",
            ".lego/certificates/h3.selfmade4u.de.key",
        )
        .await
        .unwrap();
    */
    //let addr = SocketAddr::from(([127, 0, 0, 1], 8443));
    //println!("listening on {}", addr);
    /* axum_server::bind_rustls(addr, config)
    .serve(app.into_make_service())
    .await
    .unwrap();*/
    /*axum_server::bind_openssl(addr, config)
    .serve(app.into_make_service())
    .await
    .unwrap();*/

    // TODO FIXME for every accepted connection trace
    // https://opentelemetry.io/docs/specs/semconv/attributes-registry/network/

    let mut make_service = app.into_make_service_with_connect_info::<SocketAddr>();

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let mut _accept: (TcpStream, SocketAddr);

    // https://github.com/tokio-rs/axum/blob/af13c539386463b04b82f58155ee04702527212b/axum/src/serve.rs#L279

    // tell the connections to shutdown
    let (shutdown_tx, shutdown_rx) = watch::channel(());
    let shutdown_tx = Arc::new(shutdown_tx);

    // wait for the connections to finish shutdown
    let (closed_tx, closed_rx) = watch::channel(());

    info!("started up server...");

    #[allow(clippy::redundant_pub_crate)]
    loop {
        select! {
            accept = listener.accept() => {

                let (socket, remote_addr) = accept.unwrap();

                // We don't need to call `poll_ready` because `IntoMakeServiceWithConnectInfo` is always
                // ready.
                let tower_service = unwrap_infallible(make_service.call(remote_addr).await);

                let child_span = tracing::debug_span!("child");

                let shutdown_tx = Arc::clone(&shutdown_tx);
                let closed_rx = closed_rx.clone();

                tokio::spawn(async move {
                    let socket = TokioIo::new(socket);

                    let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
                        tower_service.clone().oneshot(request)
                    });

                    let builder = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new());
                    let connection = builder.serve_connection_with_upgrades(socket, hyper_service);
                    pin_mut!(connection);

                    // TODO FIXME https://github.com/tokio-rs/axum/blob/main/axum/src/serve.rs#L279 maybe its more performance to store and pin_mut!?

                    loop {
                        select! {
                            connection_result = connection.as_mut() => {
                                if let Err(err) = connection_result
                                {
                                    eprintln!("failed to serve connection: {err:#}");
                                }
                                break; // (gracefully) finished connection
                            }
                            () = shutdown_tx.closed() => {
                                println!("signal received, shutting down");
                                connection.as_mut().graceful_shutdown();
                            }
                        }
                    }
                    println!("gracefully shut down");

                    tracing::info!("hi");

                    drop(closed_rx);
                }.instrument(child_span.or_current()));
            }
            () = shutdown_signal() => {
                // TODO FIXME "graceful shutdown"
                warn!("SHUTDOWN");
                drop(shutdown_rx); // initiate shutdown
                drop(closed_rx);
                // should we drop the tcp listener here? (write a test)
                closed_tx.closed().await;
                break;
            }
        }
    }

    Ok(())
}

fn unwrap_infallible<T>(result: Result<T, Infallible>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => match err {},
    }
}

#[allow(clippy::redundant_pub_crate)]
async fn shutdown_signal() {
    // check which of these two signals we need
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
