use std::convert::Infallible;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::LazyLock;
use std::time::Duration;

use bytes::{Buf, Bytes};
use headers::{CacheControl, ContentType, ETag, HeaderMapExt, IfNoneMatch};
use http::{Response, StatusCode};
use http_body::Body;
use http_body_util::{Empty, Full};

use crate::error::AppError;
use crate::{either_http_body, ResponseTypedHeaderExt as _};

// add watcher and then use websocket to hot reload on client?
// or for dev simply enforce unbundled development where chrome directly modifies the files
// so maybe simply don't implement watcher at all

either_http_body!(either EitherBody 1 2);

pub static INDEX_CSS: &[u8] = include_bytes!("../../../frontend/bundle.css");

pub static INDEX_CSS_VERSION: LazyLock<u64> = LazyLock::new(|| {
    let mut hasher = DefaultHasher::new();
    INDEX_CSS.hash(&mut hasher);
    hasher.finish()
});

#[expect(clippy::needless_pass_by_value)]
pub fn indexcss(
    request: hyper::Request<
        impl http_body::Body<Data = impl Buf, Error = impl Into<AppError>> + Send + 'static,
    >,
) -> hyper::Response<impl Body<Data = Bytes, Error = Infallible> + Send + 'static> {
    let if_none_match: Option<IfNoneMatch> = request.headers().typed_get();
    let etag_string = "\"xyzzy\"";
    let etag = etag_string.parse::<ETag>().unwrap();
    if if_none_match.map_or(true, |h| h.precondition_passes(&etag)) {
        Response::builder()
            .status(StatusCode::OK)
            .typed_header(ContentType::from(mime::TEXT_CSS_UTF_8))
            .typed_header(etag)
            .typed_header(
                CacheControl::new()
                    .with_immutable()
                    .with_public()
                    .with_max_age(Duration::from_secs(31_536_000)),
            )
            .body(EitherBody::Option1(Full::new(Bytes::from_static(
                INDEX_CSS,
            ))))
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .body(EitherBody::Option2(Empty::default()))
            .unwrap()
    }
}
