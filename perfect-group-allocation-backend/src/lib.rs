extern crate alloc;

// determinism?

pub mod csrf_protection;
pub mod either;
pub mod error;
//pub mod h3;
pub mod components;
pub mod routes;
pub mod session;

use core::convert::Infallible;
use std::marker::PhantomData;
use std::net::{Ipv4Addr, SocketAddrV4};

use std::pin::Pin;
use std::sync::Arc;

use bytes::{Buf, Bytes};
use error::AppError;
use futures_util::{pin_mut, Future};

use http::{HeaderName, HeaderValue, Request, Response, StatusCode};
use http_body::Body;
use http_body_util::{BodyExt, Full, Limited};
use hyper::body::Incoming;
use hyper::service::{service_fn, Service};
use hyper::Method;
use hyper_util::rt::{TokioExecutor, TokioIo};
use perfect_group_allocation_config::Config;
use perfect_group_allocation_database::{get_database_connection, Pool};
use routes::bundlecss::bundlecss;
use routes::index::index;
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
            impl http_body::Body<Data = impl Buf + Send, Error = AppError> + Send + '_,
        >,
        session: &Session,
    ) -> Result<Self, AppError> {
        let not_get_or_head =
            !(request.method() == Method::GET || request.method() == Method::HEAD);

        let expected_csrf_token = session.csrf_token();

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

// https://github.com/hyperium/hyper/blob/master/examples/service_struct_impl.rs
pub struct Svc<RequestBodyBuf: Buf + Send + 'static> {
    config: tokio::sync::watch::Receiver<Arc<Config>>,
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

// boxed improves lifetime error messages by a lot
// TODO FIXME remove heap allocation again
either_http_body!(boxed EitherBodyRouter 1 2 3 4 5 6 7 404 500);
either_future!(boxed EitherFutureRouter 1 2 3 4 5 6 7 404);

impl<
        RequestBodyBuf: Buf + Send + 'static,
        RequestBody: http_body::Body<Data = RequestBodyBuf, Error = AppError> + Send + 'static,
    > Service<Request<RequestBody>> for Svc<RequestBodyBuf>
{
    type Error = Infallible;
    type Response = Response<EitherBodyRouter>;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<RequestBody>) -> Self::Future {
        // TODO FIXME only parse cookies when needed

        println!("{} {}", req.method(), req.uri().path());

        // just store csrf token here? (static files are the only ones that theoretically don't need one)
        let session = Session::new(&req);
        let error_session = session.without_temporary_openidconnect_state(); // at some point we may also want to show the logged in user etc so just clone the whole thing
        let error_config = self.config.clone();

        let result: EitherFutureRouter<Result<Response<EitherBodyRouter>, AppError>> =
            match (req.method(), req.uri().path()) {
                (&Method::GET, "/") => {
                    let config = self.config.borrow().clone();
                    EitherFutureRouter::Option1(async move {
                        Ok(index(session, &config)
                            .await?
                            .map(EitherBodyRouter::Option1))
                    })
                }
                (&Method::GET, "/bundle.css") => EitherFutureRouter::Option2(async move {
                    Ok(bundlecss(req).map(EitherBodyRouter::Option2))
                }),
                (&Method::GET, "/list") => {
                    let pool = self.pool.clone();
                    let config = self.config.borrow().clone();
                    EitherFutureRouter::Option3(async move {
                        Ok(list(session, &config, pool)
                            .await?
                            .map(EitherBodyRouter::Option3))
                    })
                }
                (&Method::GET, "/favicon.ico") => EitherFutureRouter::Option4(async move {
                    Ok(favicon_ico(req).map(EitherBodyRouter::Option4))
                }),
                (&Method::POST, "/") => {
                    let pool = self.pool.clone();
                    let config = self.config.borrow().clone();
                    EitherFutureRouter::Option5(async move {
                        Ok(create(req, session, &config, pool)
                            .await?
                            .map(EitherBodyRouter::Option5))
                    })
                }
                (&Method::POST, "/openidconnect-login") => {
                    let config = self.config.borrow().clone();
                    EitherFutureRouter::Option6(async move {
                        Ok(openid_login(session, &config)
                            .await?
                            .map(EitherBodyRouter::Option6))
                    })
                }
                (&Method::GET, "/openidconnect-redirect") => {
                    let config = self.config.borrow().clone();
                    EitherFutureRouter::Option7(async move {
                        Ok(openid_redirect(req, session, &config)
                            .await?
                            .map(EitherBodyRouter::Option7))
                    })
                }
                (_, _) => EitherFutureRouter::Option404(async move {
                    let mut not_found =
                        Response::new(Full::new(Bytes::from_static(b"404 not found")));
                    *not_found.status_mut() = StatusCode::NOT_FOUND;
                    Ok(not_found.map(EitherBodyRouter::Option404))
                }),
            };
        // TODO FIXME don't Box for performance
        Box::pin(async move {
            match result.await {
                Ok(ok) => Ok(ok),
                Err(err) => {
                    let config = error_config.borrow().clone();
                    let response = err
                        .build_error_template(error_session, &config)
                        .await
                        .map(EitherBodyRouter::Option500);
                    Ok(response)
                }
            }
        })
        /* .map_ok(|result: Response<_>| {
            result.untyped_header(ALT_SVC, HeaderValue::from_static(ALT_SVC_HEADER))
        })*/
    }
}

pub fn setup_server<B: Buf + Send + 'static>(
    config: tokio::sync::watch::Receiver<Arc<Config>>,
) -> std::result::Result<Svc<B>, AppError> {
    info!("starting up server...");

    let current_config = config.borrow().clone();

    // this one uses parallelism for generating the index css which is highly nondeterministic
    //#[cfg(not(feature = "profiling"))]
    //initialize_index_css();

    // https://github.com/hyperium/hyper/blob/master/examples/state.rs

    let pool = get_database_connection(&current_config.database_url)?;

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
    config: tokio::sync::watch::Receiver<Arc<Config>>,
) -> Result<impl Future<Output = Result<(), AppError>>, AppError> {
    let current_config = config.borrow().clone();
    let (certs, key) = load_certs_key_pair(&current_config)?;

    // needs a service that accepts some non-controllable impl Buf
    // let http3_server = run_http3_server_s2n(config.clone())?;
    // needs a service that accepts Bytes, therefore we to create separate services
    let http2_server = run_http2_server(config, certs, key).await?;

    #[allow(clippy::redundant_pub_crate)]
    Ok(async move {
        let mut http2_server = tokio::spawn(http2_server);
        //let mut http3_server = tokio::spawn(http3_server);
        select! {
            http2_result = &mut http2_server => {
                http2_result?
                //http3_server.await?
            }
            /*http3_result = &mut http3_server => {
                http3_result??;
                http2_server.await?
            }*/
        }
    })
}

/// Outer future returns when server started listening. Inner future returns when server stopped.
#[allow(clippy::cognitive_complexity)]
pub async fn run_http2_server(
    config: tokio::sync::watch::Receiver<Arc<Config>>,
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

pub fn load_certs_key_pair(
    config: &Config,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), AppError> {
    eprintln!("{:?}", std::env::current_dir());
    let certs: Vec<CertificateDer<'static>> =
        rustls_pemfile::certs(&mut config.tls.cert.clone().as_bytes())
            .map(|value| match value {
                Ok(ok) => Ok(ok),
                Err(err) => Err(AppError::TlsCertificate(err)),
            })
            .collect::<Result<Vec<CertificateDer<'static>>, AppError>>()?;
    let key = rustls_pemfile::private_key(&mut config.tls.key.as_bytes())?.unwrap();

    Ok((certs, key))
}

// TODO FIXME make configurable
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
