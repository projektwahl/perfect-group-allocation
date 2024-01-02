use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::task::{ready, Context, Poll};

use http::{HeaderMap, HeaderName, HeaderValue, Request, Response};
use itertools::Itertools;
use opentelemetry::global;
use opentelemetry::propagation::{Extractor, Injector, TextMapPropagator as _};
use opentelemetry::trace::Tracer as _;
use pin_project::pin_project;
use tower::{Layer, Service};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

// TODO FIXME add support for http client

#[derive(Debug, Clone)]
pub struct MyTraceLayer;

impl<S> Layer<S> for MyTraceLayer {
    type Service = MyTraceService<S>;

    fn layer(&self, service: S) -> Self::Service {
        MyTraceService { inner: service }
    }
}

#[derive(Debug, Clone)]
pub struct MyTraceService<S> {
    inner: S,
}

impl<S, RequestBody, ResponseBody> Service<Request<RequestBody>> for MyTraceService<S>
where
    S: Service<Request<RequestBody>, Response = Response<ResponseBody>>,
{
    type Error = S::Error;
    type Future = MyTraceFuture<S::Future>;
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

        let response_future = {
            let _ = span.enter();
            self.inner.call(request)
        };

        MyTraceFuture {
            response_future,
            span,
        }
    }
}

#[pin_project]
pub struct MyTraceFuture<F> {
    #[pin]
    response_future: F,
    span: Span,
}

impl<F, ResponseBody, Error> Future for MyTraceFuture<F>
where
    F: Future<Output = Result<Response<ResponseBody>, Error>>,
{
    type Output = Result<Response<ResponseBody>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _ = this.span.enter();

        match ready!(this.response_future.poll(cx)) {
            Ok(mut response) => {
                // trace response
                global::get_text_map_propagator(|propagator| {
                    propagator.inject_context(
                        &this.span.context(),
                        &mut MyTraceHeaderPropagator(response.headers_mut()),
                    );
                });

                let response_headers = tracing::field::debug(response.headers());

                this.span.record("responseheaders", response_headers);

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
        if let Ok(key) = HeaderName::from_str(key) {
            if let Ok(value) = HeaderValue::from_str(&value) {
                self.0.insert(key, value);
            }
        }
    }
}

impl<'a> Extractor for MyTraceHeaderPropagator<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|value| value.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|key| key.as_str()).collect()
    }
}
