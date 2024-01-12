use std::path::Path;
use std::sync::OnceLock;

use bytes::Bytes;
use headers::{ETag, Header, IfNoneMatch};
use http::header::{ETAG, IF_NONE_MATCH};
use http::{header, Response, StatusCode};
use http_body::Body;
use http_body_util::{Empty, Full};
use perfect_group_allocation_css::index_css;

use crate::error::AppError;
use crate::session::Session;
use crate::EitherBody;

// add watcher and then use websocket to hot reload on client?
// or for dev simply enforce unbundled development where chrome directly modifies the files
// so maybe simply don't implement watcher at all

// Etag and cache busting
pub async fn indexcss(
    request: hyper::Request<impl hyper::body::Body>,
    session: Session,
) -> Result<hyper::Response<impl Body<Data = Bytes, Error = AppError>>, AppError> {
    let if_none_match =
        IfNoneMatch::decode(&mut request.headers().get_all(IF_NONE_MATCH).into_iter())?;
    let etag_string = "\"xyzzy\"";
    let etag = etag_string.parse::<ETag>().unwrap();
    if if_none_match.precondition_passes(&etag) {
        Ok(Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .header(header::ETAG, etag_string)
            .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
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
