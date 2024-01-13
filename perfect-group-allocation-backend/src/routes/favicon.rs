use std::convert::Infallible;
use std::str::FromStr;
use std::time::Duration;

use bytes::Bytes;
use headers::{CacheControl, ContentType, ETag, HeaderMapExt, IfNoneMatch};
use http::{Response, StatusCode};
use http_body::Body;
use http_body_util::{Empty, Full};

use crate::{either_http_body, ResponseTypedHeaderExt as _};

static FAVICON_ICO: &[u8] = include_bytes!("../../../frontend/favicon.ico");

either_http_body!(EitherBody 1 2);

// Etag and cache busting
#[expect(clippy::needless_pass_by_value)]
pub fn favicon_ico(
    request: hyper::Request<
        impl http_body::Body<Data = Bytes, Error = hyper::Error> + Send + 'static,
    >,
) -> hyper::Response<impl Body<Data = Bytes, Error = Infallible> + Send + 'static> {
    let if_none_match: Option<IfNoneMatch> = request.headers().typed_get();
    let etag_string = "\"xyzzy\"";
    let etag = etag_string.parse::<ETag>().unwrap();
    if if_none_match.map_or(true, |h| h.precondition_passes(&etag)) {
        Response::builder()
            .status(StatusCode::OK)
            .typed_header(ContentType::from_str("image/x-icon").unwrap())
            .typed_header(etag)
            .typed_header(
                CacheControl::new()
                    .with_immutable()
                    .with_public()
                    .with_max_age(Duration::from_secs(31_536_000)),
            )
            .body(EitherBody::Option1(Full::new(Bytes::from_static(
                FAVICON_ICO,
            ))))
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .body(EitherBody::Option2(Empty::default()))
            .unwrap()
    }
}
