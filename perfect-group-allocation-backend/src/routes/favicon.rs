use axum::response::IntoResponse;
use axum_extra::{headers, TypedHeader};
use http::{header, StatusCode};

use crate::session::Session;

static FAVICON_ICO: &[u8] = include_bytes!("../../../frontend/favicon.ico");

// Etag and cache busting
pub async fn favicon_ico(
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
                    (header::CONTENT_TYPE, "image/x-icon"),
                ],
                FAVICON_ICO,
            )
                .into_response(),
        )
    } else {
        (session, StatusCode::NOT_MODIFIED.into_response())
    }
}
