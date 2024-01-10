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

// determinism?

pub mod csrf_protection;
pub mod error;
mod openid;
pub mod routes;
//pub mod session;

use core::convert::Infallible;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use error::AppError;
use futures_util::{pin_mut, Future};
use http::{Request, Response, StatusCode};
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::service::Service;
use hyper::Method;
use hyper_util::rt::{TokioExecutor, TokioIo};
use perfect_group_allocation_database::{get_database_connection, Pool};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::watch;
use tracing::{error, info, warn};

pub trait CsrfSafeExtractor {}

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
    pub value: T,
}
/*
impl<T> FromRequest<MyState> for CsrfSafeForm<T>
where
    T: DeserializeOwned + CsrfToken + Send,
{
    type Rejection = AppError;

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

        #[expect(clippy::disallowed_types, reason = "this is the csrf safe wrapper")]
        let extractor = axum::Form::<T>::from_request(req, state).await?;

        if not_get_or_head {
            let actual_csrf_token = extractor.0.csrf_token();

            if expected_csrf_token != actual_csrf_token {
                return Err(AppError::WrongCsrfToken);
            }
        }
        Ok(Self { value: extractor.0 })
    }
}
*/
impl<T: CsrfToken> CsrfSafeExtractor for CsrfSafeForm<T> {}

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

/*
#[tokio::main]
async fn main() -> Result<(), AppError> {
    // avoid putting more code here as this is outside of all spans so doesn't get traced
    #[cfg(feature = "perfect-group-allocation-telemetry")]
    let _guard = setup_telemetry();
    #[cfg(feature = "perfect-group-allocation-telemetry")]
    tokio_runtime_metrics();

    program().await
}
*/

// https://github.com/hyperium/hyper/blob/master/examples/service_struct_impl.rs
struct Svc {
    pool: Pool,
}

impl Svc {
    async fn call(&self, req: Request<hyper::body::Incoming>) -> Response<Full<Bytes>> {
        let connection = self.pool.get().await.unwrap();

        Response::new(Full::new(Bytes::from_static(b"hi")))
    }
}

pub async fn setup_server(database_url: &str) -> std::result::Result<Svc, AppError> {
    info!("starting up server...");

    // this one uses parallelism for generating the index css which is highly nondeterministic
    //#[cfg(not(feature = "profiling"))]
    //initialize_index_css();
    #[cfg(not(feature = "profiling"))]
    openid::initialize_openid_client().await;

    // https://github.com/hyperium/hyper/blob/master/examples/state.rs

    let pool = get_database_connection(database_url)?;

    //let service = ServeDir::new("frontend");

    //.route(&Method::GET, "/", index)
    //.route(&Method::POST, "/", create)
    //.route(&Method::GET, "/index.css", indexcss)
    //.route(&Method::GET, "/favicon.ico", favicon_ico)
    //.route(&Method::GET, "/list", list)
    //.route(&Method::GET, "/download", handler)
    //.route(&Method::POST, "/openidconnect-login", openid_login)
    //.route(&Method::GET, "/openidconnect-redirect", openid_redirect);

    let app = Svc { pool };

    //let app = app.layer(CatchPanicLayer::new());
    #[cfg(feature = "perfect-group-allocation-telemetry")]
    let app = app.layer(perfect_group_allocation_telemetry::trace_layer::MyTraceLayer);
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

    Ok(app)
}

//#[cfg_attr(feature = "perfect-group-allocation-telemetry", tracing::instrument)]

#[allow(clippy::cognitive_complexity)]
pub async fn run_server(
    database_url: String,
) -> Result<impl Future<Output = Result<(), AppError>>, AppError> {
    let mut service = setup_server(&database_url).await?;

    // https://github.com/hyperium/hyper/blob/master/examples/graceful_shutdown.rs

    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 3000))
        .await
        .unwrap();

    // tell the connections to shutdown
    let (shutdown_tx, shutdown_rx) = watch::channel(());
    let shutdown_tx = Arc::new(shutdown_tx);

    // wait for the connections to finish shutdown
    let (closed_tx, closed_rx) = watch::channel(());

    info!("started up server...");

    Ok(async move {
        #[allow(clippy::redundant_pub_crate)]
        loop {
            select! {
                accept = listener.accept() => {
                    // TODO FIXME don't unwrap
                    let (socket, remote_addr) = accept.unwrap();

                    let shutdown_tx = Arc::clone(&shutdown_tx);
                    let closed_rx = closed_rx.clone();

                    let fut = async move {
                        let socket = TokioIo::new(socket);

                        let builder = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new());
                        let connection = builder.serve_connection_with_upgrades(socket, service);
                        pin_mut!(connection);

                        loop {
                            select! {
                                connection_result = connection.as_mut() => {
                                    if let Err(err) = connection_result
                                    {
                                        error!("failed to serve connection: {err:#}");
                                    }
                                    break; // (gracefully) finished connection
                                }
                                () = shutdown_tx.closed() => {
                                    connection.as_mut().graceful_shutdown();
                                }
                            }
                        }

                        tracing::info!("hi");

                        drop(closed_rx);
                    };

                    // to create a connection span we think we need this manual connection implementation

                    #[cfg(feature = "perfect-group-allocation-telemetry")]
                    let child_span = tracing::debug_span!("child");
                    #[cfg(feature = "perfect-group-allocation-telemetry")]
                    let fut = tracing::Instrument::instrument(fut, child_span.or_current());
                    tokio::spawn(fut);
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
    })
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
