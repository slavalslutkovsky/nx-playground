//! Health endpoints and DLQ admin API
//!
//! Provides K8s-ready health probes and admin endpoints for DLQ management.

use crate::consumer::StreamConsumer;
use crate::dlq::{DlqEntry, DlqManager, DlqStats};
use crate::metrics::render_metrics;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::error;

/// Shared state for health endpoints
#[derive(Clone)]
pub struct HealthState {
    redis: Arc<ConnectionManager>,
    app_name: &'static str,
    app_version: &'static str,
    stream_name: String,
}

impl HealthState {
    /// Create new HealthState
    pub fn new(
        redis: Arc<ConnectionManager>,
        app_name: &'static str,
        app_version: &'static str,
        stream_name: impl Into<String>,
    ) -> Self {
        Self {
            redis,
            app_name,
            app_version,
            stream_name: stream_name.into(),
        }
    }
}

/// Health check response
#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    app: &'static str,
    version: &'static str,
}

/// Readiness check response
#[derive(Serialize)]
struct ReadinessResponse {
    status: &'static str,
    redis: &'static str,
}

/// Stream info response
#[derive(Serialize)]
struct StreamInfoResponse {
    stream_name: String,
    length: i64,
    pending_count: i64,
    consumer_group: String,
}

/// Create the health router
///
/// Endpoints:
/// - `GET /health`, `/healthz` - Liveness probe
/// - `GET /ready`, `/readyz` - Readiness probe (checks Redis)
/// - `GET /stream/info` - Stream statistics
/// - `GET /metrics` - Prometheus metrics
pub fn health_router(state: HealthState) -> Router {
    Router::new()
        .route("/health", get(liveness))
        .route("/healthz", get(liveness))
        .route("/ready", get(readiness))
        .route("/readyz", get(readiness))
        .route("/stream/info", get(stream_info))
        .route("/metrics", get(metrics_handler))
        .with_state(state)
}

/// Create full admin router with DLQ endpoints
///
/// Includes all health endpoints plus:
/// - `GET /admin/dlq/stats` - DLQ statistics
/// - `GET /admin/dlq/entries` - List DLQ entries
/// - `GET /admin/dlq/entries/{id}` - Get specific entry
/// - `POST /admin/dlq/reprocess/{id}` - Reprocess entry
/// - `DELETE /admin/dlq/entries/{id}` - Delete entry
/// - `DELETE /admin/dlq/purge` - Purge all entries
pub fn full_admin_router(state: HealthState) -> Router {
    Router::new()
        .route("/health", get(liveness))
        .route("/healthz", get(liveness))
        .route("/ready", get(readiness))
        .route("/readyz", get(readiness))
        .route("/stream/info", get(stream_info))
        .route("/metrics", get(metrics_handler))
        .route("/admin/dlq/stats", get(dlq_stats))
        .route("/admin/dlq/entries", get(dlq_list))
        .route("/admin/dlq/entries/{id}", get(dlq_get).delete(dlq_delete))
        .route("/admin/dlq/reprocess/{id}", post(dlq_reprocess))
        .route("/admin/dlq/purge", delete(dlq_purge))
        .with_state(state)
}

// Liveness probe
async fn liveness(State(state): State<HealthState>) -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok",
        app: state.app_name,
        version: state.app_version,
    })
}

// Readiness probe
async fn readiness(State(state): State<HealthState>) -> impl IntoResponse {
    let mut conn = (*state.redis).clone();

    // Try a PING command
    let redis_ok = redis::cmd("PING")
        .query_async::<String>(&mut conn)
        .await
        .is_ok();

    if redis_ok {
        (
            StatusCode::OK,
            Json(ReadinessResponse {
                status: "ready",
                redis: "connected",
            }),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ReadinessResponse {
                status: "not_ready",
                redis: "disconnected",
            }),
        )
    }
}

// Stream info endpoint
async fn stream_info(State(state): State<HealthState>) -> impl IntoResponse {
    use crate::config::WorkerConfig;

    let config = WorkerConfig::new(&state.stream_name, "temp_group");
    let consumer = StreamConsumer::new(state.redis.clone(), config);

    match consumer.stream_info().await {
        Ok(info) => (
            StatusCode::OK,
            Json(StreamInfoResponse {
                stream_name: info.stream_name,
                length: info.length,
                pending_count: info.pending_count,
                consumer_group: info.consumer_group,
            }),
        ),
        Err(e) => {
            error!("Failed to get stream info: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(StreamInfoResponse {
                    stream_name: state.stream_name.clone(),
                    length: -1,
                    pending_count: -1,
                    consumer_group: "error".to_string(),
                }),
            )
        }
    }
}

// Prometheus metrics endpoint
async fn metrics_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("content-type", "text/plain; charset=utf-8")],
        render_metrics(),
    )
}

// DLQ stats endpoint
async fn dlq_stats(State(state): State<HealthState>) -> impl IntoResponse {
    let dlq_stream = format!("{}:dlq", state.stream_name.trim_end_matches(":jobs"));
    let dlq = DlqManager::new(state.redis.clone(), dlq_stream);

    match dlq.stats().await {
        Ok(stats) => (StatusCode::OK, Json(stats)),
        Err(e) => {
            error!("Failed to get DLQ stats: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DlqStats {
                    stream_name: "error".to_string(),
                    length: -1,
                    oldest_entry_id: None,
                    newest_entry_id: None,
                }),
            )
        }
    }
}

/// Query parameters for DLQ list
#[derive(Deserialize)]
struct DlqListQuery {
    count: Option<usize>,
    offset: Option<String>,
}

// DLQ list endpoint
async fn dlq_list(
    State(state): State<HealthState>,
    Query(query): Query<DlqListQuery>,
) -> impl IntoResponse {
    let dlq_stream = format!("{}:dlq", state.stream_name.trim_end_matches(":jobs"));
    let dlq = DlqManager::new(state.redis.clone(), dlq_stream);

    let count = query.count.unwrap_or(20);

    match dlq.list(count, query.offset.as_deref()).await {
        Ok(entries) => (StatusCode::OK, Json(entries)),
        Err(e) => {
            error!("Failed to list DLQ entries: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![] as Vec<DlqEntry>))
        }
    }
}

// DLQ get entry endpoint
async fn dlq_get(
    State(state): State<HealthState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let dlq_stream = format!("{}:dlq", state.stream_name.trim_end_matches(":jobs"));
    let dlq = DlqManager::new(state.redis.clone(), dlq_stream);

    match dlq.get(&id).await {
        Ok(Some(entry)) => (StatusCode::OK, Json(Some(entry))),
        Ok(None) => (StatusCode::NOT_FOUND, Json(None)),
        Err(e) => {
            error!("Failed to get DLQ entry: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(None))
        }
    }
}

// DLQ delete entry endpoint
async fn dlq_delete(
    State(state): State<HealthState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let dlq_stream = format!("{}:dlq", state.stream_name.trim_end_matches(":jobs"));
    let dlq = DlqManager::new(state.redis.clone(), dlq_stream);

    match dlq.delete(&id).await {
        Ok(true) => StatusCode::NO_CONTENT,
        Ok(false) => StatusCode::NOT_FOUND,
        Err(e) => {
            error!("Failed to delete DLQ entry: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// Reprocess response
#[derive(Serialize)]
struct ReprocessResponse {
    message: String,
    new_stream_id: Option<String>,
}

// DLQ reprocess endpoint (placeholder - requires producer)
async fn dlq_reprocess(
    State(state): State<HealthState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let dlq_stream = format!("{}:dlq", state.stream_name.trim_end_matches(":jobs"));
    let dlq = DlqManager::new(state.redis.clone(), dlq_stream);

    match dlq.get(&id).await {
        Ok(Some(_entry)) => {
            // TODO: Actually requeue the job
            // This requires access to the producer
            (
                StatusCode::OK,
                Json(ReprocessResponse {
                    message: "Reprocess not yet implemented".to_string(),
                    new_stream_id: None,
                }),
            )
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ReprocessResponse {
                message: "Entry not found".to_string(),
                new_stream_id: None,
            }),
        ),
        Err(e) => {
            error!("Failed to get DLQ entry for reprocess: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ReprocessResponse {
                    message: format!("Error: {}", e),
                    new_stream_id: None,
                }),
            )
        }
    }
}

/// Purge response
#[derive(Serialize)]
struct PurgeResponse {
    deleted_count: i64,
}

// DLQ purge endpoint
async fn dlq_purge(State(state): State<HealthState>) -> impl IntoResponse {
    let dlq_stream = format!("{}:dlq", state.stream_name.trim_end_matches(":jobs"));
    let dlq = DlqManager::new(state.redis.clone(), dlq_stream);

    match dlq.purge().await {
        Ok(count) => (StatusCode::OK, Json(PurgeResponse { deleted_count: count })),
        Err(e) => {
            error!("Failed to purge DLQ: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(PurgeResponse { deleted_count: -1 }))
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_health_state_creation() {
        // Just a compile test - real tests need Redis
    }
}
