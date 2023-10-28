use core::any::Any;
use core::panic::AssertUnwindSafe;
use core::task::Poll;

use anyhow::anyhow;
use axum::http::{self, HeaderValue};
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

fn log(value: &Box<dyn Any + Send + 'static>) -> anyhow::Error {
    value.downcast_ref::<&'static str>().map_or_else(
        || {
            value.downcast_ref::<String>().map_or_else(
                || anyhow!("unknown panic {:?}", (**value).type_id()),
                |string| anyhow!("{}", string),
            )
        },
        |str_slice| anyhow!("{}", str_slice),
    )
}

impl<S> Service<axum::http::Request<axum::body::Body>> for CatchPanicMiddleware<S>
where
    S: Service<axum::http::Request<axum::body::Body>, Response = Response> + Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Sync + Send + 'static>>,
    S::Future: Send + 'static,
{
    // it needs to be sync for some reason
    type Error = Box<dyn std::error::Error + Sync + Send + 'static>;
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
    type Response = S::Response;

    fn poll_ready(&mut self, cx: &mut core::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(core::convert::Into::into)
    }

    // TODO FIXME maybe we could return an Err here, then let it get traced, then convert to 500
    fn call(&mut self, request: axum::http::Request<axum::body::Body>) -> Self::Future {
        let request_id = request.headers().get("x-request-id").map_or_else(
            || String::from("hi"),
            |h| h.to_str().unwrap_or_default().to_string(),
        );
        let Ok(future) = std::panic::catch_unwind(AssertUnwindSafe(|| self.inner.call(request)))
        else {
            return Box::pin(async move {
                let text_plain: HeaderValue = HeaderValue::from_static("text/plain; charset=utf-8");

                let mut res = Response::new(Full::from(format!(
                    "an unexpected internal error occured. to report this error, specify the \
                     following request id: {request_id}",
                )));
                *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

                #[allow(clippy::declare_interior_mutable_const)]
                res.headers_mut()
                    .insert(http::header::CONTENT_TYPE, text_plain);

                Ok(res.map(|body| body.map_err(axum::Error::new).boxed_unsync()))
            });
        };
        Box::pin(async move {
            match AssertUnwindSafe(future).catch_unwind().await {
                Ok(response) => response.map_err(core::convert::Into::into),
                Err(err) => {
                    /*let mut res = Response::new(Full::from(format!(
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
                        */
                    //let err: Box<dyn std::error::Error + std::marker::Send + Sync + 'static> =
                    //    anyhow!("test").into();

                    // argument panic was called with, usually string
                    Err(log(&err).into())
                    //return Ok(res.map(|body| body.map_err(|v| axum::Error::new(v)).boxed_unsync()));
                }
            }
        })
    }
}
