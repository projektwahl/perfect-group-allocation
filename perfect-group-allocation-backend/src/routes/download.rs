use axum::response::IntoResponse;
use hyper::header;
use tokio_util::io::ReaderStream;

use crate::error::AppError;
use crate::session::Session;

pub async fn handler(session: Session) -> Result<impl IntoResponse, AppError> {
    let file = tokio::fs::File::open("/var/cache/pacman/pkg/notfound.zst").await?;
    let stream = ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);

    let headers = [
        (header::CONTENT_TYPE, "application/octet-stream"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"firefox-118.0.2-1-x86_64.pkg.tar.zst\"",
        ),
    ];

    Ok((headers, body))
}
