#[axum::debug_handler(body=MyBody, state=MyState)]
async fn handler(mut stream: BodyStream) -> Result<impl IntoResponse, AppError> {
    while let Some(_chunk) = stream.try_next().await? {}
    let file = tokio::fs::File::open("/var/cache/pacman/pkg/firefox-118.0.2-1-x86_64.pkg.tar.zst")
        .await
        .unwrap();
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
}
