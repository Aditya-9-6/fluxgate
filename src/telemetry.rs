use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use tracing::info;
use opentelemetry::global;
use metrics_exporter_prometheus::PrometheusBuilder;

pub fn init_telemetry() -> anyhow::Result<()> {
    // 1. Initialize OpenTelemetry Jaeger exporter
    global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("fluxgate-proxy")
        .install_simple()?;

    // 2. Setup Tracing Subscriber with JSON formatting + OTel layer
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Switch to JSON logging as per requirements
    let fmt_layer = fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(telemetry_layer)
        .with(fmt_layer)
        .init();

    // 3. Initialize Prometheus Metrics Exporter 
    // This runs a background HTTP server on port 9090 exposing /metrics
    PrometheusBuilder::new()
        .with_http_listener(([0, 0, 0, 0], 9090))
        .install()
        .map_err(|e| anyhow::anyhow!("Failed to install Prometheus builder: {}", e))?;

    info!("📊 [METRICS] Prometheus Exporter initialized on :9090/metrics");
    Ok(())
}
