use std::collections::HashMap;

use http::{HeaderMap, HeaderValue};
use opentelemetry::baggage::BaggageExt as _;
use opentelemetry::propagation::{Extractor, Injector, TextMapPropagator as _};
use opentelemetry::trace::{TraceContextExt as _, Tracer as _, TracerProvider as _};
use opentelemetry::KeyValue;
use opentelemetry_sdk::propagation::{
    BaggagePropagator, TextMapCompositePropagator, TraceContextPropagator,
};
use opentelemetry_sdk::trace::TracerProvider;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnResponse, MakeSpan, OnRequest, OnResponse, TraceLayer,
};
use tracing::Level;
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

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
