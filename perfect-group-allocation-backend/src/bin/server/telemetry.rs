use opentelemetry::global::logger_provider;
use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

// https://github.com/open-telemetry/opentelemetry-rust/commit/897e70a0936f11efcc05cfc9c342891fb2976f35

fn setup_tracing(resource: Resource) {
    const DEFAULT_LOG_LEVEL: &str = "trace,tokio=debug,runtime=debug,hyper=info,reqwest=info,\
                                     h2=info,tower=info,tonic=info,tower_http=trace";

    // will also redirect log events to trace events
    let stdout_log = tracing_subscriber::fmt::layer().pretty();

    let tracing_layer = tracing_opentelemetry::layer().with_tracer(
        opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint("http://localhost:4317"),
            )
            .with_trace_config(opentelemetry_sdk::trace::config().with_resource(resource))
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .unwrap(),
    );

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
        .init();
}

fn setup_logging(resource: Resource) {
    let logger = opentelemetry_otlp::new_pipeline()
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
}

fn setup_metrics(resource: Resource) {
    let meter_provider = opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_resource(resource)
        .build()
        .unwrap();
}

pub fn setup_telemetry() {
    let resource = opentelemetry_sdk::Resource::new(vec![KeyValue::new(
        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
        "perfect-group-allocation",
    )]);

    setup_tracing(resource.clone());
    setup_logging(resource.clone());
    setup_metrics(resource);
}
