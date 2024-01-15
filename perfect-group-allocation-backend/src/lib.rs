#![feature(gen_blocks)]
#![feature(lint_reasons)]
#![feature(let_chains)]
#![feature(hash_raw_entry)]
#![feature(impl_trait_in_assoc_type)]
#![feature(error_generic_member_access)]
#![feature(try_blocks)]
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
pub mod either;
pub mod error;
pub mod h3;
pub mod routes;
pub mod session;

use core::convert::Infallible;
use std::marker::PhantomData;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;

use bytes::{Buf, Bytes};
use cookie::Cookie;
use error::AppError;
use futures_util::{pin_mut, Future, FutureExt, TryFutureExt};
use h3::run_http3_server_s2n;
use http::header::{ALT_SVC, COOKIE};
use http::{HeaderName, HeaderValue, Request, Response, StatusCode};
use http_body::Body;
use http_body_util::{BodyExt, Full, Limited};
use hyper::body::Incoming;
use hyper::service::{service_fn, Service};
use hyper::Method;
use hyper_util::rt::{TokioExecutor, TokioIo};
use perfect_group_allocation_config::Config;
use perfect_group_allocation_database::{get_database_connection, Pool};
use routes::index::index;
use routes::indexcss::indexcss;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use session::Session;
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::watch;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
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

impl<T> CsrfSafeForm<T>
where
    T: DeserializeOwned + CsrfToken + Send,
{
    async fn from_request(
        request: hyper::Request<
            impl http_body::Body<Data = impl Buf + Send, Error = AppError> + Send + 'static,
        >,
        session: &Session,
    ) -> Result<Self, AppError> {
        let not_get_or_head =
            !(request.method() == Method::GET || request.method() == Method::HEAD);

        let expected_csrf_token = session.session().0;

        let body: Bytes = Limited::new(request.into_body(), 100)
            .collect()
            .await
            .unwrap()
            .to_bytes();

        let extractor: T = serde_urlencoded::from_bytes(&body).unwrap();

        if not_get_or_head {
            let actual_csrf_token = extractor.csrf_token();

            if expected_csrf_token != actual_csrf_token {
                return Err(AppError::WrongCsrfToken);
            }
        }
        Ok(Self { value: extractor })
    }
}

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

use headers::{Header, HeaderMapExt};

use crate::routes::favicon::favicon_ico;
use crate::routes::openid_login::openid_login;
use crate::routes::openid_redirect::openid_redirect;
use crate::routes::projects::create::create;
use crate::routes::projects::list::list;

pub trait ResponseTypedHeaderExt {
    #[must_use]
    fn typed_header<H: Header>(self, header: H) -> Self;

    #[must_use]
    fn untyped_header(self, key: HeaderName, value: HeaderValue) -> Self;
}

impl ResponseTypedHeaderExt for hyper::http::response::Builder {
    fn typed_header<H: Header>(mut self, header: H) -> Self {
        if let Some(res) = self.headers_mut() {
            res.typed_insert(header);
        }
        self
    }

    fn untyped_header(self, key: HeaderName, value: HeaderValue) -> Self {
        self.header(key, value)
    }
}

impl<T> ResponseTypedHeaderExt for Response<T> {
    fn typed_header<H: Header>(mut self, header: H) -> Self {
        self.headers_mut().typed_insert(header);
        self
    }

    fn untyped_header(mut self, key: HeaderName, value: HeaderValue) -> Self {
        self.headers_mut().append(key, value);
        self
    }
}

// Yieldok a value.
#[macro_export]
macro_rules! yieldfv {
    ($e:expr) => {{
        let expr = $e;
        let value = expr.1;
        let ret = expr.0;
        yield Ok::<::http_body::Frame<::bytes::Bytes>, ::core::convert::Infallible>(
            ::http_body::Frame::data(match value {
                ::alloc::borrow::Cow::Owned(v) => ::bytes::Bytes::from(v),
                ::alloc::borrow::Cow::Borrowed(v) => ::bytes::Bytes::from(v),
            }),
        );
        ret
    }};
}

/// Yieldok an iterator.
#[macro_export]
macro_rules! yieldfi {
    ($e:expr) => {{
        let expr = $e;
        let mut iterator = expr.1;
        let ret = expr.0;
        loop {
            let value = ::std::iter::Iterator::next(&mut iterator);
            // maybe match has bad liveness analysis?
            if value.is_some() {
                let value = value.unwrap();
                yield Ok::<::http_body::Frame<::bytes::Bytes>, ::core::convert::Infallible>(
                    ::http_body::Frame::data(::bytes::Bytes::from(value)),
                );
            } else {
                break;
            }
        }
        ret
    }};
}

// https://github.com/hyperium/hyper/blob/master/examples/service_struct_impl.rs
pub struct Svc<RequestBodyBuf: Buf + Send + 'static> {
    config: Config,
    pool: Pool,
    phantom_data: PhantomData<RequestBodyBuf>,
}

impl<RequestBodyBuf: Buf + Send + 'static> Clone for Svc<RequestBodyBuf> {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            pool: self.pool.clone(),
            phantom_data: self.phantom_data,
        }
    }
}

pub fn get_session<T>(request: &Request<T>) -> Session {
    let cookies = request
        .headers()
        .get_all(COOKIE)
        .into_iter()
        .filter_map(|value| value.to_str().ok())
        .map(std::borrow::ToOwned::to_owned)
        .flat_map(Cookie::split_parse)
        .filter_map(std::result::Result::ok);
    let mut jar = cookie::CookieJar::new();
    for cookie in cookies {
        jar.add_original(cookie);
    }

    Session::new(jar)
}

either_http_body!(EitherBodyRouter 1 2 3 4 5 6 7 404 500);
either_future!(EitherFutureRouter 1 2 3 4 5 6 7 404);

impl<
    RequestBodyBuf: Buf + Send + 'static,
    RequestBody: http_body::Body<Data = RequestBodyBuf, Error = AppError> + Send + 'static,
> Service<Request<RequestBody>> for Svc<RequestBodyBuf>
{
    type Error = Infallible;
    type Response = Response<impl http_body::Body<Data = Bytes, Error = Infallible> + Send>;

    type Future = impl Future<Output = Result<Self::Response, Self::Error>> + Send + 'static;

    fn call(&self, req: Request<RequestBody>) -> Self::Future {
        // TODO FIXME only parse cookies when needed

        println!("{} {}", req.method(), req.uri().path());

        match (req.method(), req.uri().path()) {
            (&Method::GET, "/") => EitherFutureRouter::Option1(async move {
                let session = get_session(&req);
                (
                    try { index(req).await?.map(EitherBodyRouter::Option1) },
                    session,
                )
            }),
            (&Method::GET, "/index.css") => EitherFutureRouter::Option2(async move {
                let session = get_session(&req);
                (
                    try { indexcss(req).map(EitherBodyRouter::Option2) },
                    session,
                )
            }),
            (&Method::GET, "/list") => {
                let pool = self.pool.clone();
                EitherFutureRouter::Option3(async move {
                    let session = get_session(&req);
                    (
                        try { list(req, pool).await?.map(EitherBodyRouter::Option3) },
                        session,
                    )
                })
            }
            (&Method::GET, "/favicon.ico") => EitherFutureRouter::Option4(async move {
                let session = get_session(&req);
                (
                    try { favicon_ico(req).map(EitherBodyRouter::Option4) },
                    session,
                )
            }),
            (&Method::POST, "/") => {
                let pool = self.pool.clone();
                EitherFutureRouter::Option5(async move {
                    let session = get_session(&req);
                    (
                        try { create(req, pool).await?.map(EitherBodyRouter::Option5) },
                        session,
                    )
                })
            }
            (&Method::POST, "/openidconnect-login") => {
                let config = self.config.clone();
                EitherFutureRouter::Option6(async move {
                    let session = get_session(&req);
                    (
                        try {
                            openid_login(req, config)
                                .await?
                                .map(EitherBodyRouter::Option6)
                        },
                        session,
                    )
                })
            }
            (&Method::GET, "/openidconnect-redirect") => {
                let config = self.config.clone();
                EitherFutureRouter::Option7(async move {
                    let mut session = get_session(&req);
                    let result = openid_redirect(req, &mut session, config)
                        .await
                        .map(|v| v.map(EitherBodyRouter::Option7));
                    (result, session)
                })
            }
            (_, _) => EitherFutureRouter::Option404(async move {
                let session = get_session(&req);
                let mut not_found = Response::new(Full::new(Bytes::from_static(b"404 not found")));
                *not_found.status_mut() = StatusCode::NOT_FOUND;
                (try { not_found.map(EitherBodyRouter::Option404) }, session)
            }),
        }
        .map(|fut: (Result<_, AppError>, Session)| match fut {
            (Ok(ok), session) => Ok(ok),
            (Err(err), session) => {
                // TODO FIXME this may need to set a cookief
                Ok(err
                    .build_error_template(session)
                    .map(EitherBodyRouter::Option500))
            }
        })
        .map_ok(|result: Response<_>| {
            result.untyped_header(ALT_SVC, HeaderValue::from_static(ALT_SVC_HEADER))
        })
    }
}

pub fn setup_server<B: Buf + Send + 'static>(
    config: Config,
) -> std::result::Result<Svc<B>, AppError> {
    info!("starting up server...");

    // this one uses parallelism for generating the index css which is highly nondeterministic
    //#[cfg(not(feature = "profiling"))]
    //initialize_index_css();

    // https://github.com/hyperium/hyper/blob/master/examples/state.rs

    let pool = get_database_connection(&config.database_url)?;

    //let service = ServeDir::new("frontend");

    //.route(&Method::GET, "/", index)
    //.route(&Method::POST, "/", create)
    //.route(&Method::GET, "/index.css", indexcss)
    //.route(&Method::GET, "/favicon.ico", favicon_ico)
    //.route(&Method::GET, "/list", list)
    //.route(&Method::GET, "/download", handler)
    //.route(&Method::POST, "/openidconnect-login", openid_login)
    //.route(&Method::GET, "/openidconnect-redirect", openid_redirect);

    let app = Svc {
        config,
        pool,
        phantom_data: PhantomData,
    };

    //let app = app.layer(CatchPanicLayer::new());
    #[cfg(feature = "perfect-group-allocation-telemetry")]
    let app = app.layer(perfect_group_allocation_telemetry::trace_layer::MyTraceLayer);

    Ok(app)
}

//#[cfg_attr(feature = "perfect-group-allocation-telemetry", tracing::instrument)]

pub async fn setup_http2_http3_server(
    config: Config,
) -> Result<impl Future<Output = Result<(), AppError>>, AppError> {
    let (certs, key) = load_certs_key_pair()?;

    // needs a service that accepts some non-controllable impl Buf
    let http3_server = run_http3_server_s2n(config.clone())?;
    // needs a service that accepts Bytes, therefore we to create separate services
    let http2_server = run_http2_server(config, certs, key).await?;

    #[allow(clippy::redundant_pub_crate)]
    Ok(async move {
        let mut http2_server = tokio::spawn(http2_server);
        let mut http3_server = tokio::spawn(http3_server);
        select! {
            http2_result = &mut http2_server => {
                http2_result??;
                http3_server.await?
            }
            http3_result = &mut http3_server => {
                http3_result??;
                http2_server.await?
            }
        }
    })
}

/// Outer future returns when server started listening. Inner future returns when server stopped.
#[allow(clippy::cognitive_complexity)]
pub async fn run_http2_server(
    config: Config,
    certs: Vec<CertificateDer<'static>>, // TODO FIXME put these into the config file
    key: PrivateKeyDer<'static>,
) -> Result<impl Future<Output = Result<(), AppError>>, AppError> {
    // https://github.com/hyperium/hyper/blob/master/examples/graceful_shutdown.rs
    let service = setup_server(config)?;

    let incoming = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), PORT))
        .await
        .unwrap();

    let mut server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec(), b"http/1.0".to_vec()];

    let tls_acceptor = TlsAcceptor::from(Arc::new(server_config));

    // tell the connections to shutdown
    let (shutdown_tx, shutdown_rx) = watch::channel(());
    let shutdown_tx = Arc::new(shutdown_tx);

    // wait for the connections to finish shutdown
    let (closed_tx, closed_rx) = watch::channel(());

    info!("started up server...");

    #[allow(clippy::redundant_pub_crate)]
    Ok(async move {
        loop {
            select! {
                incoming_accept = incoming.accept() => {
                    // TODO FIXME don't unwrap
                    let (tcp_stream, _remote_addr) = incoming_accept.unwrap();

                    let tls_acceptor = tls_acceptor.clone();

                    let shutdown_tx = Arc::clone(&shutdown_tx);
                    let closed_rx = closed_rx.clone();

                    let service = service.clone();

                    let fut = async move {
                        let tls_stream = match tls_acceptor.accept(tcp_stream).await {
                            Ok(tls_stream) => tls_stream,
                            Err(err) => {
                                eprintln!("failed to perform tls handshake: {err:#}");
                                return;
                            }
                        };
                        let builder = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new());
                        let socket = TokioIo::new(tls_stream);
                        let connection = builder.serve_connection_with_upgrades(socket, service_fn(|req| service.call(req.map(|body: Incoming| body.map_err(AppError::from)))));
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

fn load_certs(filename: &str) -> std::io::Result<Vec<CertificateDer>> {
    // TODO FIXME async
    let certfile = std::fs::File::open(filename)?;
    let mut reader = std::io::BufReader::new(certfile);

    rustls_pemfile::certs(&mut reader).collect()
}

// Load private key from file.
fn load_private_key(filename: &str) -> std::io::Result<PrivateKeyDer> {
    let keyfile = std::fs::File::open(filename)?;
    let mut reader = std::io::BufReader::new(keyfile);

    // TODO FIXME remove unwrap
    Ok(rustls_pemfile::private_key(&mut reader)?.unwrap())
}

pub fn load_certs_key_pair()
-> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), AppError> {
    let certs = load_certs(CERT_PATH)?;
    let key = load_private_key(KEY_PATH)?;

    Ok((certs, key))
}

pub const CERT_PATH: &str = ".lego/certificates/h3.selfmade4u.de.crt";
pub const KEY_PATH: &str = ".lego/certificates/h3.selfmade4u.de.key";
pub const PORT: u16 = 443;
pub const ALT_SVC_HEADER: &str = r#"h3=":443"; ma=2592000; persist=1"#;

#[allow(clippy::redundant_pub_crate)]
async fn shutdown_signal() {
    std::future::pending::<()>().await;
    /*
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
    }*/
}
