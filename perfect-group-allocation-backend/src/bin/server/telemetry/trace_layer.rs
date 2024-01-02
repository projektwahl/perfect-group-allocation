use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use http::{HeaderMap, HeaderValue};
use opentelemetry::baggage::BaggageExt as _;
use opentelemetry::propagation::{Extractor, Injector, TextMapPropagator as _};
use opentelemetry::trace::{TraceContextExt as _, Tracer as _, TracerProvider as _};
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::propagation::{
    BaggagePropagator, TextMapCompositePropagator, TraceContextPropagator,
};
use opentelemetry_sdk::trace::TracerProvider;
use pin_project::pin_project;
use tokio::time::Sleep;
use tower::Service;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnResponse, MakeSpan, OnRequest, OnResponse, TraceLayer,
};
use tracing::Level;
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

impl<S, Request> Service<Request> for TraceService<S>
where
    S: Service<Request>,
    S::Error: Into<BoxError>,
{
    type Error = BoxError;
    type Future = TraceFuture<S::Future>;
    type Response = S::Response;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let context = global::get_text_map_propagator(|propagator| {
            propagator.extract_with_context(
                &opentelemetry::Context::current(),
                &MyTraceHeaderPropagator(request.headers_mut()),
            )
        });

        let span = tracing::debug_span!(
            "request",
            method = %request.method(),
            uri = %request.uri(),
            version = ?request.version(),
            headers = ?request.headers(),
        );

        span.set_parent(context);

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

impl<F, Response, Error> Future for TraceFuture<F>
where
    F: Future<Output = Result<Response, Error>>,
    Error: Into<BoxError>,
{
    type Output = Result<Response, BoxError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match this.response_future.poll(cx) {
            Poll::Ready(result) => {
                let result = result.map_err(Into::into);

                global::get_text_map_propagator(|propagator| {
                    propagator.inject_context(
                        &opentelemetry::Context::current(),
                        &mut MyTraceHeaderPropagator(response.headers_mut()),
                    )
                });

                let response_headers = tracing::field::debug(response.headers());

                tracing::debug!(
                    latency = latency.as_nanos(),
                    response_headers,
                    "finished processing request"
                );

                return Poll::Ready(result);
            }
            Poll::Pending => {}
        }

        Poll::Pending
    }
}

#[derive(Debug, Default)]
struct TraceError(());

impl fmt::Display for TraceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("request timed out")
    }
}

impl std::error::Error for TraceError {}

type BoxError = Box<dyn std::error::Error + Send + Sync>;

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
