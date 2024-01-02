use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::task::{ready, Context, Poll};

use http::{HeaderMap, HeaderValue, Request, Response};
use opentelemetry::global;
use opentelemetry::propagation::{Extractor, Injector, TextMapPropagator as _};
use opentelemetry::trace::Tracer as _;
use pin_project::pin_project;
use tower::Service;
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

#[derive(Debug, Clone)]
struct TraceService<S> {
    inner: S,
}

impl<S> TraceService<S> {
    fn new(inner: S) -> Self {
        TraceService { inner }
    }
}

impl<S, RequestBody, ResponseBody> Service<Request<RequestBody>> for TraceService<S>
where
    S: Service<Request<RequestBody>, Response = Response<ResponseBody>>,
{
    type Error = S::Error;
    type Future = TraceFuture<S::Future>;
    type Response = S::Response;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, mut request: Request<RequestBody>) -> Self::Future {
        let context = global::get_text_map_propagator(|propagator| {
            propagator.extract(&MyTraceHeaderPropagator(request.headers_mut()))
        });

        // TODO FIXME set way more values
        let span = tracing::debug_span!(
            "request",
            method = %request.method(),
            uri = %request.uri(),
            version = ?request.version(),
            headers = ?request.headers(),
        );

        span.set_parent(context);

        // TODO FIXME async span needs correct handling

        tracing::debug!("started processing request");

        let response_future = self.inner.call(request);

        TraceFuture { response_future }
    }
}

#[pin_project]
struct TraceFuture<F> {
    #[pin]
    response_future: F,
}

impl<F, ResponseBody, Error> Future for TraceFuture<F>
where
    F: Future<Output = Result<Response<ResponseBody>, Error>>,
{
    type Output = Result<Response<ResponseBody>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match ready!(this.response_future.poll(cx)) {
            Ok(mut response) => {
                // trace response
                global::get_text_map_propagator(|propagator| {
                    propagator.inject_context(
                        &opentelemetry::Context::current(),
                        &mut MyTraceHeaderPropagator(response.headers_mut()),
                    )
                });

                let response_headers = tracing::field::debug(response.headers());

                tracing::debug!(response_headers, "finished processing request");

                Poll::Ready(Ok(response))
            }
            Err(error) => {
                // TODO trace error
                Poll::Ready(Err(error))
            }
        }
    }
}

// https://github.com/slickbench/tower-opentelemetry/blob/main/src/lib.rs
// https://github.com/mattiapenati/tower-otel/blob/main/src/trace/http.rs
// https://docs.rs/tracing-opentelemetry/latest/tracing_opentelemetry/#special-fields
// https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/blob/main/tracing-opentelemetry-instrumentation-sdk/src/http/http_server.rs

pub struct MyTraceHeaderPropagator<'a>(&'a mut HeaderMap<HeaderValue>);

impl<'a> Injector for MyTraceHeaderPropagator<'a> {
    fn set(&mut self, key: &str, value: String) {
        todo!()
    }
}

impl<'a> Extractor for MyTraceHeaderPropagator<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        todo!()
    }

    fn keys(&self) -> Vec<&str> {
        todo!()
    }
}
