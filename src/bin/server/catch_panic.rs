use std::panic::AssertUnwindSafe;
use std::task::{Context, Poll};

use axum::http::{self, HeaderValue, Request};
use axum::response::Response;
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use http_body::{Body, Full};
use hyper::StatusCode;
use tower::{Layer, Service};

#[derive(Clone)]
pub struct CatchPanicLayer;

impl<S> Layer<S> for CatchPanicLayer {
    type Service = CatchPanicMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CatchPanicMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct CatchPanicMiddleware<S> {
    inner: S,
}

impl<S> Service<Request<axum::body::Body>> for CatchPanicMiddleware<S>
where
    S: Service<Request<axum::body::Body>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Error = S::Error;
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
    type Response = S::Response;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    // TODO FIXME maybe we could return an Err here, then let it get traced, then convert to 500
    fn call(&mut self, request: Request<axum::body::Body>) -> Self::Future {
        let request_id = request
            .headers()
            .get("x-request-id")
            .map(|h| h.to_str().unwrap_or_default().to_string())
            .unwrap_or(String::from("hi"));
        let Ok(future) = std::panic::catch_unwind(AssertUnwindSafe(|| self.inner.call(request)))
        else {
            return Box::pin(async move {
                let mut res = Response::new(Full::from(format!(
                    "an unexpected internal error occured. to report this error, specify the \
                     following request id: {}",
                    request_id
                )));
                *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

                #[allow(clippy::declare_interior_mutable_const)]
                const TEXT_PLAIN: HeaderValue =
                    HeaderValue::from_static("text/plain; charset=utf-8");
                res.headers_mut()
                    .insert(http::header::CONTENT_TYPE, TEXT_PLAIN);

                Ok(res.map(|body| body.map_err(|v| axum::Error::new(v)).boxed_unsync()))
            });
        };
        Box::pin(async move {
            let Ok(response) = AssertUnwindSafe(future).catch_unwind().await else {
                let mut res = Response::new(Full::from(format!(
                    "an unexpected internal error occured. to report this error, specify the \
                     following request id: {}",
                    request_id
                )));
                *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

                #[allow(clippy::declare_interior_mutable_const)]
                const TEXT_PLAIN: HeaderValue =
                    HeaderValue::from_static("text/plain; charset=utf-8");
                res.headers_mut()
                    .insert(http::header::CONTENT_TYPE, TEXT_PLAIN);

                return Ok(res.map(|body| body.map_err(|v| axum::Error::new(v)).boxed_unsync()));
            };
            response
        })
    }
}
