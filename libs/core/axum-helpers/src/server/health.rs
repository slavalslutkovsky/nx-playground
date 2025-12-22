use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use core_config::AppInfo;
use futures::future::join_all;
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub name: &'static str,
    pub version: &'static str,
}

#[derive(Serialize)]
pub struct ReadyResponse {
    pub ready: bool,
    pub services: ServiceStatus,
}

#[derive(Serialize)]
pub struct ServiceStatus {
    pub database: bool,
    pub cache: bool,
}

/// A boxed future for health checks with a string error
pub type HealthCheckFuture<'a> = Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>>;

/// Runs multiple health checks concurrently and returns aggregated results.
///
/// # Arguments
/// * `checks` - Vector of (name, check_future) tuples
///
/// # Example
/// ```ignore
/// let checks = vec![
///     ("database", Box::pin(async {
///         check_database(db).await.map_err(|e| e.to_string())
///     })),
///     ("redis", Box::pin(async {
///         check_redis(redis).await.map_err(|e| e.to_string())
///     })),
/// ];
/// run_health_checks(checks).await
/// ```
pub async fn run_health_checks(
    checks: Vec<(&str, HealthCheckFuture<'_>)>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Run all checks concurrently
    let names: Vec<_> = checks.iter().map(|(name, _)| *name).collect();
    let futures: Vec<_> = checks.into_iter().map(|(_, check)| check).collect();
    let results = join_all(futures).await;

    // Aggregate results
    let mut status_map = HashMap::new();
    let mut all_healthy = true;

    for (name, result) in names.into_iter().zip(results) {
        match result {
            Ok(_) => {
                status_map.insert(name, "connected");
            }
            Err(e) => {
                tracing::error!("Readiness check failed: {} error: {:?}", name, e);
                status_map.insert(name, "disconnected");
                all_healthy = false;
            }
        }
    }

    let mut response = json!({
        "status": if all_healthy { "ready" } else { "not ready" }
    });

    // Add each check result to the response
    if let Value::Object(ref mut map) = response {
        for (name, status) in status_map {
            map.insert(name.to_string(), json!(status));
        }
    }

    if all_healthy {
        Ok((StatusCode::OK, Json(response)))
    } else {
        Err((StatusCode::SERVICE_UNAVAILABLE, Json(response)))
    }
}

/// Health check endpoint handler.
///
/// Returns a simple health status response with app name and version.
/// This endpoint should always return 200 if the service is running.
pub async fn health_handler(State(app): State<AppInfo>) -> Response {
    let response = HealthResponse {
        status: "healthy",
        name: app.name,
        version: app.version,
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// Creates a router with the /health endpoint.
///
/// Use this to add liveness checks to your app. The handler returns
/// the app name and version from `AppInfo`.
///
/// # Example
/// ```ignore
/// use axum_helpers::health::health_router;
/// use core_config::app_info;
///
/// let app_info = app_info!();
/// let app = Router::new()
///     .merge(health_router(app_info))
///     .merge(ready_router(state));
/// ```
pub fn health_router(app_info: AppInfo) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .with_state(app_info)
}
