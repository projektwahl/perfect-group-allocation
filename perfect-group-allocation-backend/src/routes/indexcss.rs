use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

use bytes::Bytes;
use headers::{CacheControl, ContentType, ETag, Header, HeaderMapExt, IfNoneMatch};
use http::header::{ETAG, IF_NONE_MATCH};
use http::{header, Response, StatusCode};
use http_body::Body;
use http_body_util::{Empty, Full};
use perfect_group_allocation_css::index_css;

use crate::error::AppError;
use crate::session::Session;
use crate::{EitherBody, ResponseTypedHeaderExt as _};

// add watcher and then use websocket to hot reload on client?
// or for dev simply enforce unbundled development where chrome directly modifies the files
// so maybe simply don't implement watcher at all

pub fn indexcss(
    request: hyper::Request<hyper::body::Incoming>,
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
            .typed_header(ContentType::from(mime::TEXT_CSS_UTF_8))
            .typed_header(etag)
            .typed_header(
                CacheControl::new()
                    .with_immutable()
                    .with_public()
                    .with_max_age(Duration::from_secs(31_536_000)),
            )
            .body(EitherBody::Left(Full::new(Bytes::from_static(
                index_css!(),
            ))))
            .unwrap())
    } else {
        Ok(Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .body(EitherBody::Right(Empty::default()))
            .unwrap())
    }
}
