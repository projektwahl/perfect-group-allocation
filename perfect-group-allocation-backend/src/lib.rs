#![feature(gen_blocks)]
#![feature(lint_reasons)]
#![feature(let_chains)]
#![feature(hash_raw_entry)]
#![feature(impl_trait_in_assoc_type)]
#![feature(error_generic_member_access)]
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
pub mod routes;
pub mod session;

use core::convert::Infallible;
use std::marker::PhantomData;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::Path;
use std::sync::Arc;

use bytes::{Buf, Bytes};
use cookie::Cookie;
use error::AppError;
use futures_util::{pin_mut, Future, FutureExt, StreamExt, TryFutureExt};
use h3::error::{Code, ErrorLevel};
use http::header::{ALT_SVC, COOKIE};
use http::{HeaderName, HeaderValue, Request, Response, StatusCode};
use http_body::{Body, Frame};
use http_body_util::{BodyExt, Full, Limited};
use hyper::body::Incoming;
use hyper::service::{service_fn, Service};
use hyper::Method;
use hyper_util::rt::{TokioExecutor, TokioIo};
use itertools::Itertools;
use perfect_group_allocation_database::{get_database_connection, Pool};
use pin_project::pin_project;
use routes::index::index;
use routes::indexcss::indexcss;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use session::Session;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::{join, select};
use tokio_rustls::rustls::version::TLS13;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
use tokio_rustls::TlsAcceptor;
use tracing::{error, info, trace_span, warn};

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
            impl http_body::Body<Data = impl Buf, Error = AppError> + Send + 'static,
        >,
        session: Session,
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
use zero_cost_templating::async_iterator_extension::AsyncIterExt;

use crate::routes::favicon::favicon_ico;
use crate::routes::openid_login::openid_login;
use crate::routes::projects::create::create;
use crate::routes::projects::list::list;

pub trait ResponseTypedHeaderExt {
    #[must_use]
    fn typed_header<H: Header>(self, header: H) -> Self;

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
    pool: Pool,
    phantom_data: PhantomData<RequestBodyBuf>,
}

impl<RequestBodyBuf: Buf + Send + 'static> Clone for Svc<RequestBodyBuf> {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            phantom_data: self.phantom_data,
        }
    }
}

either_http_body!(EitherBodyRouter 1 2 3 4 5 6 7 8);
either_future!(EitherFutureRouter 1 2 3 4 5 6 7);

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
        let cookies = req
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

        let session = Session::new(jar);
        let err_session = session.clone(); // TODO FIXME

        println!("{} {}", req.method(), req.uri().path());

        match (req.method(), req.uri().path()) {
            (&Method::GET, "/") => EitherFutureRouter::Option1(async move {
                Ok(index(session).await?.map(EitherBodyRouter::Option1))
            }),
            (&Method::GET, "/index.css") => EitherFutureRouter::Option2(async move {
                Ok(indexcss(req).map(EitherBodyRouter::Option2))
            }),
            (&Method::GET, "/list") => {
                let pool = self.pool.clone();
                EitherFutureRouter::Option3(async move {
                    Ok(list(pool, session).await?.map(EitherBodyRouter::Option3))
                })
            }
            (&Method::GET, "/favicon.ico") => EitherFutureRouter::Option4(async move {
                Ok(favicon_ico(req).map(EitherBodyRouter::Option4))
            }),
            (&Method::POST, "/") => {
                let pool = self.pool.clone();
                EitherFutureRouter::Option5(async move {
                    Ok(create(req, pool, session)
                        .await?
                        .map(EitherBodyRouter::Option5))
                })
            }
            (&Method::GET, "/openidconnect-login") => EitherFutureRouter::Option6(async move {
                Ok(openid_login(session).await?.map(EitherBodyRouter::Option6))
            }),
            (_, _) => EitherFutureRouter::Option7(async move {
                let mut not_found = Response::new(Full::new(Bytes::from_static(b"404 not found")));
                *not_found.status_mut() = StatusCode::NOT_FOUND;
                Ok(not_found.map(EitherBodyRouter::Option7))
            }),
        }
        .map(|fut: Result<_, AppError>| match fut {
            Ok(ok) => Ok(ok),
            Err(err) => Ok(err
                .build_error_template(err_session)
                .map(EitherBodyRouter::Option8)),
        })
        .map_ok(|result: Response<_>| {
            result.untyped_header(ALT_SVC, HeaderValue::from_static(ALT_SVC_HEADER))
        })
    }
}

pub fn setup_server<B: Buf + Send + 'static>(
    database_url: &str,
) -> std::result::Result<Svc<B>, AppError> {
    info!("starting up server...");

    // this one uses parallelism for generating the index css which is highly nondeterministic
    //#[cfg(not(feature = "profiling"))]
    //initialize_index_css();

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

    let app = Svc {
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
    database_url: String,
) -> Result<impl Future<Output = Result<(), AppError>>, AppError> {
    let (certs, key) = load_certs_key_pair()?;

    // needs a service that accepts some non-controllable impl Buf
    let http3_server = run_http3_server_s2n(database_url.clone(), certs.clone(), key.clone())?;
    // needs a service that accepts Bytes, therefore we to create separate services
    let http2_server = run_http2_server(database_url, certs, key).await?;

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
    database_url: String,
    certs: Vec<Certificate>,
    key: PrivateKey,
) -> Result<impl Future<Output = Result<(), AppError>>, AppError> {
    // https://github.com/hyperium/hyper/blob/master/examples/graceful_shutdown.rs
    let service = setup_server(&database_url)?;

    let incoming = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), PORT))
        .await
        .unwrap();

    let mut server_config = ServerConfig::builder()
        .with_safe_defaults()
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
                        let connection = builder.serve_connection_with_upgrades(socket, service_fn(|req| service.call(req.map(|body: Incoming| body.map_err(|e| AppError::from(e))))));
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

static ALPN: &[u8] = b"h3";

pub struct H3Body<S: h3::quic::RecvStream + 'static>(h3::server::RequestStream<S, Bytes>);

impl<S: h3::quic::RecvStream + 'static> Body for H3Body<S> {
    type Error = h3::Error;

    type Data = impl Buf + Send + 'static;

    fn poll_frame(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let recv_data = self.0.recv_data();
        let recv_data = std::pin::pin!(recv_data);
        recv_data
            .poll(cx)
            .map(|v| v.transpose().map(|v| v.map(|v| Frame::data(v))))
    }
}

fn load_certs(filename: &str) -> std::io::Result<Vec<Certificate>> {
    // TODO FIXME async
    // Open certificate file.
    let certfile = std::fs::File::open(filename)?;
    let mut reader = std::io::BufReader::new(certfile);

    // Load and return certificate.
    let certs = rustls_pemfile::certs(&mut reader)?;
    Ok(certs.into_iter().map(Certificate).collect())
}

// Load private key from file.
fn load_private_key(filename: &str) -> std::io::Result<PrivateKey> {
    // Open keyfile.
    let keyfile = std::fs::File::open(filename)?;
    let mut reader = std::io::BufReader::new(keyfile);

    // Load and return a single private key.
    let keys = rustls_pemfile::ec_private_keys(&mut reader)?;

    Ok(PrivateKey(keys[0].clone()))
}

pub fn load_certs_key_pair() -> Result<(Vec<Certificate>, PrivateKey), AppError> {
    let certs = load_certs(CERT_PATH)?;
    let key = load_private_key(KEY_PATH)?;

    Ok((certs, key))
}

const CERT_PATH: &str = ".lego/certificates/h3.selfmade4u.de.crt";
const KEY_PATH: &str = ".lego/certificates/h3.selfmade4u.de.key";
const PORT: u16 = 443;
const ALT_SVC_HEADER: &str = r#"h3=":443"; ma=2592000; persist=1"#;

async fn handle_connection<C: h3::quic::Connection<Bytes> + Send, MyRecvStream: h3::quic::RecvStream + 'static>(
service: Svc::<<H3Body<MyRecvStream> as Body>::Data>, // the service needs to use the same impl buf that recvstream decided to use
    mut connection: h3::server::Connection<C, Bytes>,
) where
// RequestBodyBuf needs to == H3Body::Data

// the RecvStream uses impl Buf
    C::BidiStream: h3::quic::BidiStream<Bytes, RecvStream = MyRecvStream> + Send + 'static,
    <C as h3::quic::Connection<bytes::Bytes>>::SendStream: Send,
    <C as h3::quic::Connection<bytes::Bytes>>::RecvStream: Send,
    <<C as h3::quic::Connection<bytes::Bytes>>::BidiStream as h3::quic::BidiStream<bytes::Bytes>>::RecvStream: std::marker::Send,
    <<C as h3::quic::Connection<bytes::Bytes>>::BidiStream as h3::quic::BidiStream<bytes::Bytes>>::SendStream: std::marker::Send
{
    loop {
        match connection.accept().await {
            Ok(Some((req, stream))) => {
                info!("new request: {:#?}", req);

                let service = service.clone();

                tokio::spawn(async move {
                    let (mut response_body, mut request_body) = stream.split();
                    let request = req.map(|_| H3Body(request_body));
                    let response = service
                        .call(request.map(|body| body.map_err(|e| AppError::from(e))))
                        .await;
                    match response {
                        Ok(response) => {
                            let response: Response<_> = response;
                            let (parts, body) = response.into_parts();

                            response_body
                                .send_response(Response::from_parts(parts, ()))
                                .await
                                .unwrap();

                            let mut body = std::pin::pin!(body);
                            while let Some(value) = body.frame().await {
                                let value = value.unwrap();
                                if value.is_data() {
                                    response_body
                                        .send_data(value.into_data().unwrap())
                                        .await
                                        .unwrap();
                                } else if value.is_trailers() {
                                    response_body
                                        .send_trailers(value.into_trailers().unwrap())
                                        .await
                                        .unwrap();
                                    return;
                                }
                            }
                            response_body.finish().await.unwrap();
                        }
                        Err(error) => {
                            let _error: Infallible = error;
                        }
                    }
                });
            }

            // indicating no more streams to be received
            Ok(None) => {
                break;
            }

            Err(err) => {
                error!("error on accept {}", err);
                match err.get_error_level() {
                    ErrorLevel::ConnectionError => break,
                    ErrorLevel::StreamError => continue,
                }
            }
        }
    }
}

type TestS2n = <H3Body<s2n_quic_h3::RecvStream> as Body>::Data;

pub fn run_http3_server_s2n(
    database_url: String,
    certs: Vec<Certificate>,
    key: PrivateKey,
) -> Result<impl Future<Output = Result<(), AppError>>, AppError> {
    let service = setup_server::<TestS2n>(&database_url)?;

    let mut server = s2n_quic::Server::builder()
        .with_tls((Path::new(CERT_PATH), Path::new(KEY_PATH)))?
        .with_io(format!("127.0.0.1:{PORT}").as_str())?
        .start()?;

    info!("listening on localhost:{PORT}");

    // https://github.com/aws/s2n-quic/blob/main/quic/s2n-quic-qns/src/server/h3.rs

    Ok(async move {
        while let Some(mut connection) = server.accept().await {
            let service = service.clone();

            // spawn a new task for the connection
            tokio::spawn(async move {
                let mut connection =
                    h3::server::Connection::new(s2n_quic_h3::Connection::new(connection))
                        .await
                        .unwrap();
                handle_connection(service, connection).await;
            });
        }
        Ok(())
    })
}

type TestQuinn = <H3Body<h3_quinn::RecvStream> as Body>::Data;

pub fn run_http3_server_quinn(
    database_url: String,
    certs: Vec<Certificate>,
    key: PrivateKey,
) -> Result<impl Future<Output = Result<(), AppError>>, AppError> {
    let service = setup_server::<TestQuinn>(&database_url)?;

    let listen = std::net::SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), PORT));

    let mut tls_config = ServerConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[&TLS13])
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    tls_config.max_early_data_size = u32::MAX;
    tls_config.alpn_protocols = vec![ALPN.into()];

    let server_config = quinn::ServerConfig::with_crypto(Arc::new(tls_config));
    let endpoint = quinn::Endpoint::server(server_config, listen)?;

    info!("listening on {}", listen);

    Ok(async move {
        // handle incoming connections and requests

        while let Some(new_conn) = endpoint.accept().await {
            trace_span!("New connection being attempted");

            let service = service.clone();

            tokio::spawn(async move {
                match new_conn.await {
                    Ok(conn) => {
                        info!("new connection established");

                        let mut connection: h3::server::Connection<h3_quinn::Connection, Bytes> =
                            h3::server::Connection::new(h3_quinn::Connection::new(conn))
                                .await
                                .unwrap();
                        handle_connection(service, connection).await;
                    }
                    Err(err) => {
                        error!("accepting connection failed: {:?}", err);
                    }
                }
            });
        }

        // shut down gracefully
        // wait for connections to be closed before exiting
        endpoint.wait_idle().await;

        Ok(())
    })
}

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
