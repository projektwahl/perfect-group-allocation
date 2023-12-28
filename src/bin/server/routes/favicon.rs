use std::io::Read;
use std::path::Path;
use std::sync::{Once, OnceLock};

use axum::response::IntoResponse;
use axum_extra::either::Either;
use axum_extra::response::Css;
use axum_extra::{headers, TypedHeader};
use http::{header, StatusCode};
use lightningcss::bundler::{Bundler, FileProvider};
use lightningcss::stylesheet::{ParserOptions, PrinterOptions};
use lightningcss::targets::Targets;
use parcel_sourcemap::SourceMap;
use tokio::task::spawn_blocking;

use crate::error::to_error_result;
use crate::session::Session;
use crate::XRequestId;

static FAVICON_ICO: OnceLock<Vec<u8>> = OnceLock::new();

pub async fn initialize_favicon_ico() {
    FAVICON_ICO
        .set(tokio::fs::read("frontend/favicon.ico").await.unwrap())
        .unwrap();
}

// Etag and cache busting
pub async fn favicon_ico(
    TypedHeader(XRequestId(request_id)): TypedHeader<XRequestId>,
    if_none_match: TypedHeader<headers::IfNoneMatch>,
    session: Session,
) -> (Session, impl IntoResponse) {
    let etag_string = "\"xyzzy\"";
    let etag = etag_string.parse::<headers::ETag>().unwrap();
    println!("{:?}", if_none_match);

    if if_none_match.precondition_passes(&etag) {
        (
            session,
            (
                [
                    (header::ETAG, etag_string),
                    (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
                    (header::CONTENT_TYPE, "image/x-icon"),
                ],
                (&**FAVICON_ICO.get().unwrap()),
            )
                .into_response(),
        )
    } else {
        (session, StatusCode::NOT_MODIFIED.into_response())
    }
}
