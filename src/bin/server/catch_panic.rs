use std::any::{type_name_of_val, Any};
use std::fmt::Debug;
use std::panic::AssertUnwindSafe;
use std::task::{Context, Poll};

use anyhow::anyhow;
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

fn log<T: Any + Debug>(value: &T) -> anyhow::Error {
    let value_any = value as &dyn Any;

    // Try to convert our value to a `String`. If successful, we want to
    // output the `String`'s length as well as its value. If not, it's a
    // different type: just print it out unadorned.
    match value_any.downcast_ref::<String>() {
        Some(as_string) => {
            anyhow!("String ({}): {}", as_string.len(), as_string)
        }
        None => {
            anyhow!("{:?} {value:?}", type_name_of_val(value_any))
        }
    }
}

impl<S> Service<Request<axum::body::Body>> for CatchPanicMiddleware<S>
where
    S: Service<Request<axum::body::Body>, Response = Response> + Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Sync + Send + 'static>>,
    S::Future: Send + 'static,
{
    // it needs to be sync for some reason
    type Error = Box<dyn std::error::Error + Sync + Send + 'static>;
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
    type Response = S::Response;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|e| e.into())
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
            match AssertUnwindSafe(future).catch_unwind().await {
                Ok(response) => response.map_err(|e| e.into()),
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
                    return Err(log(&err).into());
                    //return Ok(res.map(|body| body.map_err(|v| axum::Error::new(v)).boxed_unsync()));
                }
            }
        })
    }
}
