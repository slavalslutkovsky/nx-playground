//! Axum middleware for automatic HTTP request metrics.

use axum::{
    body::Body,
    extract::MatchedPath,
    http::{Request, Response},
    middleware::Next,
};
use metrics::{counter, histogram};
use std::time::Instant;

/// Middleware function for recording HTTP request metrics.
///
/// Records:
/// - `http_requests_total` - Counter with method, path, status labels
/// - `http_request_duration_seconds` - Histogram with method, path labels
/// - `http_requests_errors_total` - Counter for 4xx and 5xx responses
///
/// # Example
///
/// ```rust,ignore
/// use axum::{Router, middleware};
/// use observability::middleware::metrics_middleware;
///
/// let app = Router::new()
///     .route("/", get(handler))
///     .layer(middleware::from_fn(metrics_middleware));
/// ```
pub async fn metrics_middleware(
    matched_path: Option<MatchedPath>,
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let start = Instant::now();
    let method = request.method().to_string();
    let path = matched_path
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());

    // Process the request
    let response = next.run(request).await;

    // Record metrics
    let duration = start.elapsed();
    let status = response.status().as_u16().to_string();
    let status_class = match response.status().as_u16() {
        200..=299 => "2xx",
        300..=399 => "3xx",
        400..=499 => "4xx",
        500..=599 => "5xx",
        _ => "other",
    };

    // Request counter
    counter!(
        "http_requests_total",
        "method" => method.clone(),
        "path" => path.clone(),
        "status" => status.clone(),
        "status_class" => status_class.to_string()
    )
    .increment(1);

    // Duration histogram
    histogram!(
        "http_request_duration_seconds",
        "method" => method.clone(),
        "path" => path.clone()
    )
    .record(duration.as_secs_f64());

    // Error counter (4xx and 5xx)
    if response.status().is_client_error() || response.status().is_server_error() {
        counter!(
            "http_requests_errors_total",
            "method" => method,
            "path" => path,
            "status" => status
        )
        .increment(1);
    }

    response
}

/// A simple tower Layer wrapper for the metrics middleware.
#[derive(Clone, Copy, Default)]
pub struct MetricsLayer;
