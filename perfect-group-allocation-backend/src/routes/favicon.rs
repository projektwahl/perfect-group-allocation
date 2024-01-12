use std::str::FromStr;
use std::time::Duration;

use bytes::Bytes;
use headers::{CacheControl, ContentType, ETag, Header, HeaderMapExt, IfNoneMatch};
use http::header::IF_NONE_MATCH;
use http::{header, Response, StatusCode};
use http_body::Body;
use http_body_util::{Empty, Full};

use crate::error::AppError;
use crate::session::Session;
use crate::{EitherBody, ResponseTypedHeaderExt as _};

static FAVICON_ICO: &[u8] = include_bytes!("../../../frontend/favicon.ico");

// Etag and cache busting
pub fn favicon_ico(
    request: hyper::Request<impl hyper::body::Body>,
) -> Result<hyper::Response<impl Body<Data = Bytes, Error = AppError>>, AppError> {
    let if_none_match: Option<IfNoneMatch> = request.headers().typed_get();
    let etag_string = "\"xyzzy\"";
    let etag = etag_string.parse::<ETag>().unwrap();
    if if_none_match
        .map(|h| h.precondition_passes(&etag))
        .unwrap_or(true)
    {
        Ok(Response::builder()
            .status(StatusCode::OK)
            .typed_header(ContentType::from_str("image/x-icon").unwrap())
            .typed_header(etag)
            .typed_header(
                CacheControl::new()
                    .with_immutable()
                    .with_public()
                    .with_max_age(Duration::from_secs(31_536_000)),
            )
            .body(EitherBody::Zero(Full::new(Bytes::from_static(FAVICON_ICO))))
            .unwrap())
    } else {
        Ok(Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .body(EitherBody::One(Empty::default()))
            .unwrap())
    }
}
