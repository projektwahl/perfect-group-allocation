use std::collections::HashMap;

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

pub struct MyTraceHeaderPropagator;

impl Injector for MyTraceHeaderPropagator {
    fn set(&mut self, key: &str, value: String) {
        todo!()
    }
}

impl Extractor for MyTraceHeaderPropagator {
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
        todo!()
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
        todo!()
    }
}

#[derive(Default, Clone, Copy)]
pub struct MyTraceMakeSpan;

impl<B> MakeSpan<B> for MyTraceMakeSpan {
    fn make_span(&mut self, request: &http::Request<B>) -> tracing::Span {
        let baggage_propagator = BaggagePropagator::new();
        let trace_context_propagator = TraceContextPropagator::new();

        // Then create a composite propagator
        let composite_propagator = TextMapCompositePropagator::new(vec![
            Box::new(baggage_propagator),
            Box::new(trace_context_propagator),
        ]);

        // Then for a given implementation of `Injector`
        let mut injector = HashMap::new();

        // And a given span
        let example_span = TracerProvider::default()
            .tracer("example-component")
            .start("span-name");

        // with the current context, call inject to add the headers
        composite_propagator.inject_context(
            &opentelemetry::Context::current_with_span(example_span)
                .with_baggage(vec![KeyValue::new("test", "example")]),
            &mut injector,
        );

        example_span
    }
}

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
