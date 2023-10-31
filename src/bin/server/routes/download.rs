use axum::response::IntoResponse;
use axum::TypedHeader;
use hyper::header;
use tokio_util::io::ReaderStream;

use crate::error::AppErrorWithMetadata;
use crate::session::Session;
use crate::XRequestId;

#[axum::debug_handler(body=crate::MyBody, state=crate::MyState)]
pub async fn handler(
    TypedHeader(XRequestId(request_id)): TypedHeader<XRequestId>,
    session: Session,
) -> Result<impl IntoResponse, AppErrorWithMetadata> {
    let result = async {
        let file =
            tokio::fs::File::open("/var/cache/pacman/pkg/firefox-118.0.2-1-x86_64.pkg.tar.zst")
                .await?;
        let stream = ReaderStream::new(file);
        let body = hyper::Body::wrap_stream(stream);

        let headers = [
            (header::CONTENT_TYPE, "application/octet-stream"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"firefox-118.0.2-1-x86_64.pkg.tar.zst\"",
            ),
        ];

        Ok((headers, hyper::Response::new(body)))
    };
    match result.await {
        Ok(ok) => Ok(ok),
        Err(app_error) => {
            // TODO FIXME store request id type-safe in body/session
            Err(AppErrorWithMetadata {
                session,
                request_id,
                app_error,
            })
        }
    }
}
