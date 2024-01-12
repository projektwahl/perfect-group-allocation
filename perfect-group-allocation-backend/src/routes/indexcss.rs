use std::path::Path;
use std::sync::OnceLock;

use http::{header, StatusCode};

use crate::session::Session;

// add watcher and then use websocket to hot reload on client?
// or for dev simply enforce unbundled development where chrome directly modifies the files
// so maybe simply don't implement watcher at all

// Etag and cache busting
pub async fn indexcss(
    if_none_match: TypedHeader<headers::IfNoneMatch>,
    session: Session,
) -> (Session, impl IntoResponse) {
    let etag_string = "\"xyzzy\"";
    let etag = etag_string.parse::<headers::ETag>().unwrap();
    if if_none_match.precondition_passes(&etag) {
        (
            session,
            (
                [
                    (header::ETAG, etag_string),
                    (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
                ],
                Css(INDEX_CSS.get().unwrap().as_str()),
            )
                .into_response(),
        )
    } else {
        (session, StatusCode::NOT_MODIFIED.into_response())
    }
}
