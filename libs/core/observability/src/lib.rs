//! Observability utilities for the cloud cost optimization platform.
//!
//! This crate provides:
//! - Prometheus metrics recording and export
//! - Custom metrics for pricing, resources, and API operations
//! - Axum middleware for automatic request metrics
//!
//! # Example
//!
//! ```rust,ignore
//! use observability::{init_metrics, metrics_handler, PricingMetrics};
//!
//! // Initialize metrics recorder
//! init_metrics();
//!
//! // Record pricing operations
//! PricingMetrics::record_list_prices("aws", 50, 15);
//! PricingMetrics::record_price_created("aws", "compute");
//!
//! // Add metrics endpoint to router
//! let app = Router::new()
//!     .route("/metrics", get(metrics_handler));
//! ```

pub mod middleware;
pub mod pricing;
pub mod resources;

pub use middleware::MetricsLayer;
pub use pricing::PricingMetrics;
pub use resources::ResourceMetrics;

// Re-export metrics macros for convenience
pub use metrics::{counter, gauge, histogram};

use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use once_cell::sync::OnceCell;
use tracing::info;

static METRICS_HANDLE: OnceCell<PrometheusHandle> = OnceCell::new();

/// Initialize the Prometheus metrics recorder.
///
/// This should be called once at application startup.
/// Returns the PrometheusHandle for rendering metrics.
pub fn init_metrics() -> &'static PrometheusHandle {
    METRICS_HANDLE.get_or_init(|| {
        let handle = PrometheusBuilder::new()
            .install_recorder()
            .expect("Failed to install Prometheus recorder");

        info!("Prometheus metrics recorder initialized");

        // Register metric descriptions
        register_metric_descriptions();

        handle
    })
}

/// Get the metrics handle (must call init_metrics first)
pub fn get_metrics_handle() -> Option<&'static PrometheusHandle> {
    METRICS_HANDLE.get()
}

/// Axum handler for /metrics endpoint
pub async fn metrics_handler() -> String {
    match get_metrics_handle() {
        Some(handle) => handle.render(),
        None => "# Metrics not initialized\n".to_string(),
    }
}

/// Register metric descriptions for documentation
fn register_metric_descriptions() {
    use metrics::describe_counter;
    use metrics::describe_gauge;
    use metrics::describe_histogram;

    // HTTP metrics
    describe_counter!(
        "http_requests_total",
        "Total number of HTTP requests"
    );
    describe_histogram!(
        "http_request_duration_seconds",
        "HTTP request duration in seconds"
    );
    describe_counter!(
        "http_requests_errors_total",
        "Total number of HTTP request errors"
    );

    // Pricing metrics
    describe_counter!(
        "pricing_operations_total",
        "Total pricing operations by type and provider"
    );
    describe_histogram!(
        "pricing_operation_duration_seconds",
        "Pricing operation duration in seconds"
    );
    describe_gauge!(
        "pricing_entries_total",
        "Total number of pricing entries by provider"
    );
    describe_counter!(
        "pricing_comparisons_total",
        "Total price comparison requests"
    );

    // Resource metrics
    describe_gauge!(
        "cloud_resources_total",
        "Total cloud resources by provider and type"
    );
    describe_counter!(
        "resource_sync_operations_total",
        "Resource sync operations by status"
    );
    describe_histogram!(
        "resource_sync_duration_seconds",
        "Resource sync duration in seconds"
    );

    // Collection metrics
    describe_counter!(
        "price_collection_jobs_total",
        "Total price collection jobs by status"
    );
    describe_histogram!(
        "price_collection_duration_seconds",
        "Price collection job duration"
    );
    describe_gauge!(
        "prices_collected_last_run",
        "Prices collected in the last collection run"
    );

    // Recommendation metrics
    describe_gauge!(
        "recommendations_total",
        "Total recommendations by type and status"
    );
    describe_gauge!(
        "potential_savings_usd",
        "Total potential savings in USD cents"
    );
    describe_counter!(
        "recommendations_applied_total",
        "Total recommendations applied"
    );
}
