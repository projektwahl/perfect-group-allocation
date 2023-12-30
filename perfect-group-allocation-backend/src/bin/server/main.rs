#![feature(gen_blocks)]
#![feature(lint_reasons)]
#![feature(let_chains)]
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
pub mod routes;
pub mod session;
pub mod telemetry;

use core::convert::Infallible;
use core::time::Duration;

use axum::extract::{FromRef, FromRequest};
use axum::http::{self, HeaderName, HeaderValue};
use axum::routing::{get, post};
use axum::{async_trait, RequestExt, Router};
use axum_extra::extract::cookie::Key;
use axum_extra::headers::Header;
use error::{to_error_result, AppError};
use http::StatusCode;
use hyper::Method;
use itertools::Itertools;
use opentelemetry::global::{self, logger_provider};
use opentelemetry::metrics::MeterProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{TonicExporterBuilder, WithExportConfig};
use opentelemetry_sdk::trace::Tracer;
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
use tokio::net::TcpListener;
use tower::{service_fn, ServiceBuilder};
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::request_id::{MakeRequestUuid, SetRequestIdLayer};
use tower_http::services::ServeDir;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{error, trace, Subscriber};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::filter::Filtered;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, Registry};

use crate::openid::initialize_openid_client;
use crate::routes::openid_redirect::openid_redirect;
use crate::routes::projects::list::list;

pub trait CsrfSafeExtractor {}

#[derive(Clone, FromRef)]
struct MyState {
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
        /*let request_id = req
        .extract_parts::<TypedHeader<XRequestId>>()
        .await
        .map_or("unknown-request-id".to_owned(), |header| header.0.0);*/
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

// maybe not per request csrf but per form a different csrf token that is only valid for the form as defense in depth.
/*
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
*/

// https://github.com/sunng87/handlebars-rust/tree/master/src/helpers
// https://github.com/sunng87/handlebars-rust/blob/master/src/helpers/helper_with.rs
// https://github.com/sunng87/handlebars-rust/blob/master/src/helpers/helper_lookup.rs

pub struct XRequestId(String);

static X_REQUEST_ID_HEADER_NAME: HeaderName = http::header::HeaderName::from_static("x-request-id");

impl Header for XRequestId {
    fn name() -> &'static HeaderName {
        &X_REQUEST_ID_HEADER_NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum_extra::headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values
            .exactly_one()
            .map_err(|_e| axum_extra::headers::Error::invalid())?;
        let value = value
            .to_str()
            .map_err(|_e| axum_extra::headers::Error::invalid())?;
        Ok(Self(value.to_owned()))
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        #[expect(clippy::unwrap_used, reason = "decode ensures this is unreachable")]
        let value = HeaderValue::from_str(&self.0).unwrap();

        values.extend(core::iter::once(value));
    }
}

/*
async fn handle_error_test(
    request_id: Result<TypedHeader<XRequestId>, TypedHeaderRejection>,
    err: Box<dyn std::error::Error + Sync + Send + 'static>,
) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        // intentionally not using handlebars etc to reduce amount of potentially broken code executed here
        format!(
            "Unhandled internal error for request {}: {:?}",
            request_id.map_or("unknown-request-id".to_owned(), |header| header.0.0),
            err
        ),
    )
}
*/

pub async fn get_database_connection() -> Result<DatabaseConnection, AppError> {
    let database_url = std::env::var("DATABASE_URL")?;

    let db = Database::connect(&database_url).await?;

    Ok(db)
}

fn layers(app: Router<MyState>, db: DatabaseConnection) -> Router<()> {
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
    let app: Router<()> = app.with_state(MyState {
        database: db,
        key: Key::generate(),
    });
    //let app: Router<(), MyBody0> = app.layer(PropagateRequestIdLayer::x_request_id());
    // TODO FIXME
    let app = app.layer(
        ServiceBuilder::new()
            //.layer(HandleErrorLayer::new(handle_error_test))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::default().include_headers(true))
                    .on_response(DefaultOnResponse::default().include_headers(true)),
            )
            .layer(CatchPanicLayer::new()),
    );
    let app: Router<()> = app.layer(SetRequestIdLayer::x_request_id(MakeRequestUuid));
    app
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let _guard = setup_telemetry();

    let meter = opentelemetry::global::meter("perfect-group-allocation");

    let metric_mean_poll_duration = meter.u64_gauge("gauge.mean_poll_duration").init();

    initialize_favicon_ico().await;
    initialize_index_css();
    initialize_openid_client().await;

    let _monitor = tokio_metrics::TaskMonitor::new();
    let monitor_root = tokio_metrics::TaskMonitor::new();
    let monitor_root_create = tokio_metrics::TaskMonitor::new();
    let monitor_index_css = tokio_metrics::TaskMonitor::new();
    let monitor_list = tokio_metrics::TaskMonitor::new();
    let _monitor_download = tokio_metrics::TaskMonitor::new();
    let _monitor_openidconnect_login = tokio_metrics::TaskMonitor::new();
    let _monitor_openidconnect_redirect = tokio_metrics::TaskMonitor::new();

    let handle = tokio::runtime::Handle::current();
    let runtime_monitor = tokio_metrics::RuntimeMonitor::new(&handle);

    // print task metrics every 500ms
    {
        let monitor_index_css = monitor_index_css.clone();
        let monitor_root = monitor_root.clone();
        let monitor_root_create = monitor_root_create.clone();
        let monitor_list = monitor_list.clone();
        tokio::spawn(async move {
            for (((_interval_index_css, interval_root), _interval_create), _interval_list) in
                monitor_index_css
                    .intervals()
                    .zip(monitor_root.intervals())
                    .zip(monitor_root_create.intervals())
                    .zip(monitor_list.intervals())
            {
                // pretty-print the metric interval
                // these metrics seem to work (tested using index.css spawn_blocking)

                // we can actually do this using the observable one with .intervals()
                metric_mean_poll_duration.record(
                    interval_root.mean_poll_duration().subsec_nanos().into(),
                    &[],
                );

                /*println!(
                    "GET /index.css {:?} {:?} {:?}",
                    interval_index_css.mean_poll_duration(),
                    interval_index_css.slow_poll_ratio(),
                    interval_index_css.mean_slow_poll_duration()
                );
                println!(
                    "GET / {:?} {:?} {:?}",
                    interval_root.mean_poll_duration(),
                    interval_root.slow_poll_ratio(),
                    interval_root.mean_slow_poll_duration()
                );
                println!(
                    "POST /create {:?} {:?} {:?}",
                    interval_create.mean_poll_duration(),
                    interval_create.slow_poll_ratio(),
                    interval_create.mean_slow_poll_duration()
                );
                println!(
                    "GET /list {:?} {:?} {:?}",
                    interval_list.mean_poll_duration(),
                    interval_list.slow_poll_ratio(),
                    interval_list.mean_slow_poll_duration()
                );*/
                // wait 500ms
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        });
    }

    // print runtime metrics every 500ms
    {
        tokio::spawn(async move {
            for _interval_runtime in runtime_monitor.intervals() {
                // pretty-print the metric interval
                //println!("runtime {:?}", interval_runtime.busy_ratio());

                // wait 500ms
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });
    }

    let db = get_database_connection().await?;

    let service = ServeDir::new("frontend");

    // RUST_LOG=tower_http::trace=TRACE cargo run --bin server

    // TODO FIXME use services here so we can specify the error type
    // and then we can build a layer around it
    let app: Router<MyState> = Router::new()
        .route(
            "/",
            get(move |p1, p2| monitor_root.instrument(index(p1, p2))),
        )
        .route(
            "/",
            post(move |p1, p2, p3, p4| monitor_root_create.instrument(create(p1, p2, p3, p4))),
        )
        .route(
            "/index.css",
            get(move |p1, p2, p3| monitor_index_css.instrument(indexcss(p1, p2, p3))),
        )
        .route("/favicon.ico", get(favicon_ico))
        .route(
            "/list",
            get(move |p1, p2, p3| monitor_list.instrument(list(p1, p2, p3))),
        )
        .route("/download", get(handler))
        .route("/openidconnect-login", post(openid_login))
        .route_service(
            "/test",
            service_fn(|req: http::Request<axum::body::Body>| async move {
                let body = axum::body::Body::from(format!("Hi from `{} /foo`", req.method()));
                let res = http::Response::new(body);
                Ok::<_, Infallible>(res)
            }),
        )
        .route("/openidconnect-redirect", get(openid_redirect))
        .fallback_service(service);

    let app = layers(app, db);
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
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

#[allow(clippy::redundant_pub_crate)]
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
