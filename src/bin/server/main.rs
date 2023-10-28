#![warn(
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::alloc_instead_of_core,
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::as_underscore,
    clippy::assertions_on_result_states,
    clippy::clone_on_ref_ptr,
    clippy::create_dir,
    clippy::dbg_macro,
    clippy::decimal_literal_representation,
    clippy::default_numeric_fallback,
    clippy::deref_by_slicing,
    clippy::disallowed_script_idents,
    clippy::else_if_without_else,
    clippy::empty_drop,
    clippy::empty_structs_with_brackets,
    clippy::error_impl_error,
    clippy::exit,
    clippy::expect_used,
    clippy::filetype_is_file,
    clippy::float_arithmetic,
    clippy::float_cmp_const,
    clippy::fn_to_numeric_cast_any,
    clippy::format_push_string,
    clippy::if_then_some_else_none,
    clippy::impl_trait_in_params,
    clippy::indexing_slicing,
    clippy::integer_division,
    clippy::large_include_file,
    clippy::let_underscore_must_use,
    clippy::let_underscore_untyped,
    clippy::lossy_float_literal,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::min_ident_chars,
    clippy::missing_assert_message,
    clippy::missing_asserts_for_indexing,
    clippy::mixed_read_write_in_expression,
    clippy::mod_module_files,
    clippy::modulo_arithmetic,
    clippy::multiple_inherent_impl,
    clippy::multiple_unsafe_ops_per_block,
    clippy::mutex_atomic,
    clippy::needless_raw_strings,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::partial_pub_fields,
    clippy::pattern_type_mismatch,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::redundant_type_annotations,
    clippy::ref_patterns,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_name_method,
    clippy::semicolon_inside_block,
    clippy::shadow_unrelated,
    clippy::single_char_lifetime_names,
    clippy::std_instead_of_alloc,
    clippy::std_instead_of_core,
    clippy::str_to_string,
    clippy::string_lit_chars_any,
    clippy::string_slice,
    clippy::string_to_string,
    clippy::suspicious_xor_used_as_pow,
    clippy::tests_outside_test_module,
    clippy::todo,
    clippy::try_err,
    clippy::unimplemented,
    clippy::unnecessary_self_imports,
    clippy::unneeded_field_pattern,
    clippy::unreachable,
    clippy::unseparated_literal_suffix,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::use_debug,
    clippy::verbose_file_reads,
    clippy::wildcard_enum_match_arm
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions
)]
#![feature(coroutines)]
#![feature(never_type, unwrap_infallible)]

pub mod catch_panic;
pub mod csrf_protection;
mod entities;
mod error;
mod openid;
pub mod routes;
pub mod session;

use std::borrow::Cow;
use std::convert::Infallible;
use std::fs::File;
use std::future::poll_fn;
use std::io::BufReader;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use axum::error_handling::HandleErrorLayer;
use axum::extract::rejection::TypedHeaderRejection;
use axum::extract::{FromRef, FromRequest, State};
use axum::headers::{self, Header};
use axum::http::{self, HeaderName, HeaderValue};
use axum::routing::{get, post};
use axum::{async_trait, BoxError, Form, RequestPartsExt, Router, TypedHeader};
use axum_extra::extract::cookie::Key;
use catch_panic::CatchPanicLayer;
use error::{AppError, AppErrorWithMetadata};
use futures_util::TryFutureExt;
use handlebars::Handlebars;
use http_body::Limited;
use hyper::server::accept::Accept;
use hyper::server::conn::{AddrIncoming, Http};
use hyper::{header, Method, StatusCode};
use itertools::Itertools;
use pin_project_lite::pin_project;
use routes::download::handler;
use routes::index::index;
use routes::indexcss::indexcss;
use routes::openid_login::openid_login;
use routes::openid_redirect::openid_redirect;
use routes::projects::create::create;
use routes::projects::list::list;
use rustls_pemfile::{certs, pkcs8_private_keys};
use sea_orm::{
    ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, RuntimeErr, Statement,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use session::{Session, SessionLayer};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
use tokio_rustls::TlsAcceptor;
use tower::make::MakeService;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::{MakeRequestUuid, SetRequestIdLayer};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::{
    RequestBodyTimeoutLayer, ResponseBodyTimeoutLayer, TimeoutBody, TimeoutLayer,
};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};

const DB_NAME: &str = "postgres";

pub trait CsrfSafeExtractor {}

pub struct ExtractSession<E: CsrfSafeExtractor> {
    extractor: E,
    session: Arc<Mutex<Session>>,
}

#[async_trait]
impl<S, B, T> FromRequest<S, BodyWithSession<B>> for ExtractSession<T>
where
    B: Send + 'static,
    S: Send + Sync,
    T: CsrfSafeExtractor + FromRequest<S, BodyWithSession<B>>,
{
    type Rejection = T::Rejection;

    async fn from_request(
        req: hyper::Request<BodyWithSession<B>>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();
        let session = body.session.clone();
        let extractor = T::from_request(hyper::Request::from_parts(parts, body), state).await?;
        Ok(Self { extractor, session })
    }
}

pin_project! {
    pub struct BodyWithSession<B> {
        // TODO FIXME store request id in here and maybe improve storage of session (so no arc exposed to user?)
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
    handlebars: Arc<Handlebars<'static>>,
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

pub struct EmptyBody;

#[async_trait]
impl<S, B> FromRequest<S, B> for EmptyBody
where
    B: Send + 'static,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request(_req: hyper::Request<B>, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self)
    }
}

impl CsrfSafeExtractor for EmptyBody {}

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

#[derive(Deserialize)]
pub struct CsrfSafeForm<T: CsrfToken> {
    value: T,
}

#[async_trait]
impl<T, B> FromRequest<MyState, BodyWithSession<B>> for CsrfSafeForm<T>
where
    T: DeserializeOwned + CsrfToken,
    B: http_body::Body + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = AppErrorWithMetadata;

    async fn from_request(
        req: hyper::Request<BodyWithSession<B>>,
        state: &MyState,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let request_id = parts
            .extract::<TypedHeader<XRequestId>>()
            .await
            .map_or("unknown".to_string(), |h| h.0.0);
        let handlebars = match parts
            .extract_with_state::<State<Arc<Handlebars<'static>>>, MyState>(state)
            .await
        {
            Ok(State(handlebars)) => handlebars,
            Err(infallible) => match infallible {},
        };
        let mut session = body.session.lock().await;
        let expected_csrf_token = session.session_id();
        drop(session);
        let not_get_or_head = !(parts.method == Method::GET || parts.method == Method::HEAD);

        let result = async {
            let extractor =
                Form::<T>::from_request(hyper::Request::from_parts(parts, body), state).await?;

            if not_get_or_head {
                let actual_csrf_token = extractor.0.csrf_token();

                if expected_csrf_token != actual_csrf_token {
                    return Err(AppError::WrongCsrfToken);
                }
            }
            Ok(Self { value: extractor.0 })
        };
        result
            .or_else(|app_error| async {
                // TODO FIXME store request id type-safe in body/session

                Err(AppErrorWithMetadata {
                    csrf_token: expected_csrf_token.clone(),
                    request_id,
                    handlebars,
                    app_error,
                })
            })
            .await
    }
}

impl<T: CsrfToken> CsrfSafeExtractor for CsrfSafeForm<T> {}

// TODO rtt0
fn rustls_server_config(key: impl AsRef<Path>, cert: impl AsRef<Path>) -> Arc<ServerConfig> {
    let mut key_reader = BufReader::new(File::open(key).unwrap());
    let mut cert_reader = BufReader::new(File::open(cert).unwrap());

    let key = PrivateKey(pkcs8_private_keys(&mut key_reader).unwrap().remove(0));
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

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values
            .exactly_one()
            .map_err(|_e| headers::Error::invalid())?;
        let value = value.to_str().map_err(|_e| headers::Error::invalid())?;
        Ok(Self(value.to_string()))
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        let value = HeaderValue::from_str(&self.0).unwrap();

        values.extend(std::iter::once(value));
    }
}

async fn handle_error_test(
    request_id: Result<TypedHeader<XRequestId>, TypedHeaderRejection>,
    err: Box<dyn std::error::Error + Sync + Send + 'static>,
) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        // intentionally not using handlebars etc to reduce amount of potentially broken code executed here
        format!(
            "Unhandled internal error for request {}: {:?}",
            request_id.map_or("unknown-request-id".to_string(), |h| h.0.0),
            err
        ),
    )
}

pub async fn get_database_connection() -> Result<DatabaseConnection, DbErr> {
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
    Ok(db)
}

fn layers(
    app: Router<MyState, MyBody3>,
    db: DatabaseConnection,
    handlebars: Handlebars<'static>,
) -> Router<(), MyBody0> {
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
    let app: Router<MyState, MyBody0> = app.layer(TimeoutLayer::new(Duration::from_secs(5)));
    let app: Router<MyState, MyBody0> = app.layer(SetResponseHeaderLayer::overriding(
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
    let app: Router<(), MyBody0> = app.with_state(MyState {
        database: db,
        handlebars: Arc::new(handlebars),
    });
    //let app: Router<(), MyBody0> = app.layer(PropagateRequestIdLayer::x_request_id());
    let app = app.layer(
        ServiceBuilder::new()
            .layer(HandleErrorLayer::new(handle_error_test))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::default().include_headers(true))
                    .on_response(DefaultOnResponse::default().include_headers(true)),
            )
            .layer(CatchPanicLayer),
    );
    let app: Router<(), MyBody0> = app.layer(SetRequestIdLayer::x_request_id(MakeRequestUuid));
    app
}

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    tracing_subscriber::fmt::init();

    let db = get_database_connection().await?;

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

    // RUST_LOG=tower_http::trace=TRACE cargo run --bin server
    let app: Router<MyState, MyBody> = Router::new()
        .route("/", get(index))
        .route("/", post(create))
        .route("/index.css", get(indexcss))
        .route("/list", get(list))
        .route("/download", get(handler))
        .route("/openidconnect-login", post(openid_login))
        .route("/openidconnect-redirect", get(openid_redirect))
        .fallback_service(service);

    let mut handlebars = Handlebars::new();
    handlebars.set_dev_mode(true);
    handlebars.set_strict_mode(true);
    handlebars
        .register_templates_directory(".hbs", "./templates/")
        .unwrap();

    let app = layers(app, db, handlebars);
    let mut app = app.into_make_service();

    loop {
        let stream = poll_fn(|cx| Pin::new(&mut listener).poll_accept(cx))
            .await
            .unwrap()
            .unwrap();

        let acceptor = acceptor.clone();

        let protocol = protocol.clone();

        let svc = MakeService::<_, hyper::Request<hyper::Body>>::make_service(&mut app, &stream);

        tokio::spawn(async move {
            if let Ok(stream) = acceptor.accept(stream).await {
                let _ = protocol.serve_connection(stream, svc.await.unwrap()).await;
            }
        });
    }
}
