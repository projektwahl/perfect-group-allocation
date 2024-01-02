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
use opentelemetry::KeyValue;
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
    timeout: Duration,
}

impl<S> TraceService<S> {
    fn new(inner: S, timeout: Duration) -> Self {
        TraceService { inner, timeout }
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
        let response_future = self.inner.call(request);
        let sleep = tokio::time::sleep(self.timeout);

        TraceFuture {
            response_future,
            sleep,
        }
    }
}

#[pin_project]
struct TraceFuture<F> {
    #[pin]
    response_future: F,
    #[pin]
    sleep: Sleep,
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
                return Poll::Ready(result);
            }
            Poll::Pending => {}
        }

        match this.sleep.poll(cx) {
            Poll::Ready(()) => {
                let error = Box::new(TraceError(()));
                return Poll::Ready(Err(error));
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

#[derive(Default, Clone, Copy)]
pub struct MyTraceOnRequest;

impl<B> OnRequest<B> for MyTraceOnRequest {
    fn on_request(&mut self, request: &http::Request<B>, span: &tracing::Span) {
        tracing::debug!("started processing request");
    }
}

#[derive(Default, Clone, Copy)]
pub struct MyTraceOnResponse;

impl<B> OnResponse<B> for MyTraceOnResponse {
    fn on_response(
        self,
        response: &http::Response<B>,
        latency: std::time::Duration,
        span: &tracing::Span,
    ) {
        let baggage_propagator = BaggagePropagator::new();
        let trace_context_propagator = TraceContextPropagator::new();
        let composite_propagator = TextMapCompositePropagator::new(vec![
            Box::new(baggage_propagator),
            Box::new(trace_context_propagator),
        ]);

        composite_propagator.inject_context(
            &opentelemetry::Context::current(),
            &mut MyTraceHeaderPropagator(response.headers_mut()),
        );

        let response_headers = tracing::field::debug(response.headers());

        tracing::debug!(
            latency = latency.as_nanos(),
            response_headers,
            "finished processing request"
        );
    }
}

#[derive(Default, Clone, Copy)]
pub struct MyTraceMakeSpan;

impl<B> MakeSpan<B> for MyTraceMakeSpan {
    fn make_span(&mut self, request: &http::Request<B>) -> tracing::Span {
        let baggage_propagator = BaggagePropagator::new();
        let trace_context_propagator = TraceContextPropagator::new();
        let composite_propagator = TextMapCompositePropagator::new(vec![
            Box::new(baggage_propagator),
            Box::new(trace_context_propagator),
        ]);

        let context = composite_propagator.extract_with_context(
            &opentelemetry::Context::current(),
            &MyTraceHeaderPropagator(request.headers_mut()),
        );

        let span = tracing::debug_span!(
            "request",
            method = %request.method(),
            uri = %request.uri(),
            version = ?request.version(),
            headers = ?request.headers(),
        );

        span.set_parent(context);

        span
    }
}

#[must_use]
pub fn my_trace_layer() -> TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    MyTraceMakeSpan,
    MyTraceOnRequest,
    MyTraceOnResponse,
> {
    TraceLayer::new_for_http()
        .on_request(MyTraceOnRequest)
        .on_response(MyTraceOnResponse)
        .make_span_with(MyTraceMakeSpan)
}
