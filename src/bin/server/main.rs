#![feature(gen_blocks)]
#![forbid(unsafe_code)]
#![warn(
    future_incompatible,
    let_underscore,
    nonstandard_style,
    unused,
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
    clippy::module_name_repetitions,
    clippy::print_stdout,
    reason = "not yet ready for that"
)]
#![feature(coroutines)]
#![feature(lint_reasons)]

extern crate alloc;

pub mod csrf_protection;
mod entities;
mod error;
mod openid;
pub mod routes;
pub mod session;

use alloc::borrow::Cow;
use core::convert::Infallible;
use std::time::Duration;

use axum::extract::{FromRef, FromRequest};
use axum::http::{self, HeaderName, HeaderValue};
use axum::routing::{get, post};
use axum::{async_trait, RequestExt, Router};
use axum_extra::extract::cookie::Key;
use axum_extra::headers::Header;
use axum_extra::TypedHeader;
use error::{to_error_result, AppError};
use http::StatusCode;
use hyper::Method;
use itertools::Itertools;
use routes::download::handler;
use routes::index::index;
use routes::indexcss::indexcss;
use routes::openid_login::openid_login;
use routes::projects::create::create;
use sea_orm::{
    ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, RuntimeErr, Statement,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use session::Session;
use tokio::net::TcpListener;
use tower::service_fn;
use tower_http::request_id::{MakeRequestUuid, SetRequestIdLayer};
use tower_http::services::ServeDir;

use crate::routes::openid_redirect::openid_redirect;
use crate::routes::projects::list::list;

const DB_NAME: &str = "postgres";

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
        let request_id = req
            .extract_parts::<TypedHeader<XRequestId>>()
            .await
            .map_or("unknown-request-id".to_owned(), |header| header.0.0);
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
            Err(app_error) => Err(to_error_result(session, request_id, app_error).await),
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
                Err(err) => return Err(err.into()),
                Ok(_) => {}
            }

            let url = format!("{database_url}/{DB_NAME}");
            Database::connect(&url).await?
        }
        DbBackend::Sqlite => db,
    };
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
    /*let app = app.layer(
        ServiceBuilder::new()
            .layer(HandleErrorLayer::new(handle_error_test))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::default().include_headers(true))
                    .on_response(DefaultOnResponse::default().include_headers(true)),
            )
            .layer(CatchPanicLayer::new()),
    );*/
    let app: Router<()> = app.layer(SetRequestIdLayer::x_request_id(MakeRequestUuid));
    app
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    console_subscriber::init();

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
            for (((interval_index_css, interval_root), interval_create), interval_list) in
                monitor_index_css
                    .intervals()
                    .zip(monitor_root.intervals())
                    .zip(monitor_root_create.intervals())
                    .zip(monitor_list.intervals())
            {
                // pretty-print the metric interval
                // these metrics seem to work (tested using index.css spawn_blocking)
                println!(
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
                );
                // wait 500ms
                tokio::time::sleep(Duration::from_millis(5000)).await;
            }
        });
    }

    // print runtime metrics every 500ms
    {
        tokio::spawn(async move {
            for _ in runtime_monitor.intervals() {
                // pretty-print the metric interval

                // wait 500ms
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });
    }

    //tracing_subscriber::fmt::init();

    let db = get_database_connection().await?;

    // TODO FIXME seems like TLS is a performance bottleneck. maybe either let the reverse proxy handle this or switch to openssl for now?

    let service = ServeDir::new("frontend");

    // RUST_LOG=tower_http::trace=TRACE cargo run --bin server

    // TODO FIXME use services here so we can specify the error type
    // and then we can build a layer around it
    let app: Router<MyState> = Router::new()
        .route(
            "/",
            get(move |first, second| monitor_root.instrument(index(first, second))),
        )
        .route(
            "/",
            post(move |a, b, c, d| monitor_root_create.instrument(create(a, b, c, d))),
        )
        .route(
            "/index.css",
            get(move |first, second| monitor_index_css.instrument(indexcss(first, second))),
        )
        .route(
            "/list",
            get(move |a, b, c| monitor_list.instrument(list(a, b, c))),
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
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
