#![feature(hash_raw_entry)]
pub mod router;
pub mod tokio_metrics;
pub mod trace_layer;

use std::time::Duration;

use opentelemetry::global::{self, logger_provider};
use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::metrics::MeterProvider as SdkMeterProvider;
use opentelemetry_sdk::propagation::{
    BaggagePropagator, TextMapCompositePropagator, TraceContextPropagator,
};
use opentelemetry_sdk::resource::{
    EnvResourceDetector, OsResourceDetector, ProcessResourceDetector, TelemetryResourceDetector,
};
use opentelemetry_sdk::Resource;
use tracing_opentelemetry::{MetricsLayer, OpenTelemetrySpanExt};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

// https://github.com/open-telemetry/opentelemetry-rust/commit/897e70a0936f11efcc05cfc9c342891fb2976f35

pub struct OpenTelemetryGuard {
    meter_provider: SdkMeterProvider,
}

impl Drop for OpenTelemetryGuard {
    fn drop(&mut self) {
        global::shutdown_tracer_provider();
        global::shutdown_logger_provider();
        if let Err(err) = self.meter_provider.shutdown() {}
    }
}

#[must_use]
pub fn setup_telemetry() -> OpenTelemetryGuard {
    const DEFAULT_LOG_LEVEL: &str = "trace,tokio=debug,runtime=debug,hyper=info,reqwest=info,\
                                     h2=info,tower=info,tonic=info,tower_http=trace";
    let resource = Resource::from_detectors(
        Duration::from_secs(1),
        vec![
            Box::new(OsResourceDetector),
            Box::new(ProcessResourceDetector),
            Box::new(TelemetryResourceDetector),
            Box::new(EnvResourceDetector::new()),
        ],
    )
    .merge(&opentelemetry_sdk::Resource::new(vec![KeyValue::new(
        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
        "perfect-group-allocation",
    )]));

    // will also redirect log events to trace events
    let stdout_log = tracing_subscriber::fmt::layer();

    let tracing_provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(opentelemetry_sdk::trace::config().with_resource(resource.clone()))
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap();

    let tracing_layer = tracing_opentelemetry::layer().with_tracer(tracing_provider);

    let meter_provider = opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_resource(resource.clone())
        .build()
        .unwrap();
    let opentelemetry_metrics = MetricsLayer::new(meter_provider.clone());

    tracing_subscriber::registry()
        .with(console_subscriber::spawn())
        .with(
            stdout_log.with_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| DEFAULT_LOG_LEVEL.into()),
            ),
        )
        .with(
            tracing_layer.with_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| DEFAULT_LOG_LEVEL.into()),
            ),
        )
        .with(opentelemetry_metrics)
        .init();

    let baggage_propagator = BaggagePropagator::new();
    let trace_context_propagator = TraceContextPropagator::new();
    let composite_propagator = TextMapCompositePropagator::new(vec![
        Box::new(baggage_propagator),
        Box::new(trace_context_propagator),
    ]);
    global::set_text_map_propagator(composite_propagator);

    let _logger = opentelemetry_otlp::new_pipeline()
        .logging()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_log_config(opentelemetry_sdk::logs::Config::default().with_resource(resource))
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap();

    let logger_provider = logger_provider();
    OpenTelemetryTracingBridge::new(&logger_provider);

    tracing::Span::current().set_attribute(
        opentelemetry_semantic_conventions::trace::SERVER_ADDRESS,
        "localhost",
    );

    OpenTelemetryGuard { meter_provider }
}
