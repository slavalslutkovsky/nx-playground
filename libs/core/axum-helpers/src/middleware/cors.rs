use axum::http::{HeaderName, Method};
use std::time::Duration;
use tower_http::cors::CorsLayer;

/// Creates a CORS layer with common settings for API services.
///
/// # Arguments
/// * `allowed_origin` - The allowed origin header value
///
/// # Returns
/// A configured `CorsLayer` with:
/// - Specified allowed origin
/// - Common HTTP methods (GET, POST, PUT, DELETE, PATCH, OPTIONS)
/// - Common headers (Content-Type, Authorization, Accept, Cookie, x-csrf-token)
/// - Credentials allowed
/// - 1 hour max age
pub fn create_cors_layer(allowed_origin: axum::http::HeaderValue) -> CorsLayer {
    CorsLayer::new()
        .allow_origin(allowed_origin)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::COOKIE,
            HeaderName::from_static("x-csrf-token"),
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600))
}

/// Creates a permissive CORS layer for development.
///
/// Allows any origin - **DO NOT USE IN PRODUCTION**.
pub fn create_permissive_cors_layer() -> CorsLayer {
    CorsLayer::permissive()
}
