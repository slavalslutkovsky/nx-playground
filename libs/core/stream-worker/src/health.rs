//! Health check handlers for stream workers.
//!
//! This module provides reusable Axum handlers for:
//! - Liveness probes (`/health`, `/healthz`)
//! - Readiness probes (`/ready`, `/readyz`)
//! - Stream monitoring (`/stream/info`)
//! - Prometheus metrics (`/metrics`)
//! - DLQ admin endpoints (`/admin/dlq/*`)

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::dlq::DlqManager;
use crate::metrics;

/// Shared state for health endpoints.
#[derive(Clone)]
pub struct HealthState {
    /// Redis connection for health checks.
    pub redis: Arc<ConnectionManager>,
    /// Application name.
    pub app_name: String,
    /// Application version.
    pub app_version: String,
    /// Stream name for monitoring.
    pub stream_name: String,
    /// DLQ stream name.
    pub dlq_stream_name: String,
}

impl HealthState {
    /// Create a new health state.
    pub fn new(
        redis: Arc<ConnectionManager>,
        app_name: impl Into<String>,
        app_version: impl Into<String>,
        stream_name: impl Into<String>,
    ) -> Self {
        let stream = stream_name.into();
        let dlq = format!("{}:dlq", stream);
        Self {
            redis,
            app_name: app_name.into(),
            app_version: app_version.into(),
            stream_name: stream,
            dlq_stream_name: dlq,
        }
    }

    /// Create a new health state with custom DLQ stream name.
    pub fn with_dlq_stream(
        redis: Arc<ConnectionManager>,
        app_name: impl Into<String>,
        app_version: impl Into<String>,
        stream_name: impl Into<String>,
        dlq_stream_name: impl Into<String>,
    ) -> Self {
        Self {
            redis,
            app_name: app_name.into(),
            app_version: app_version.into(),
            stream_name: stream_name.into(),
            dlq_stream_name: dlq_stream_name.into(),
        }
    }

    /// Get a DLQ manager for this state.
    pub fn dlq_manager(&self) -> DlqManager {
        DlqManager::new(
            (*self.redis).clone(),
            &self.stream_name,
            &self.dlq_stream_name,
        )
    }
}

/// Health response for liveness probes.
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Status (always "healthy" if responding).
    pub status: &'static str,
    /// Application name.
    pub name: String,
    /// Application version.
    pub version: String,
}

/// Liveness probe handler.
///
/// Always returns OK if the server is running.
/// Use this for Kubernetes liveness probes.
pub async fn health_handler(State(state): State<HealthState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        name: state.app_name,
        version: state.app_version,
    })
}

/// Readiness probe handler.
///
/// Checks if Redis is connected and ready.
/// Use this for Kubernetes readiness probes.
pub async fn ready_handler(
    State(state): State<HealthState>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let mut conn = (*state.redis).clone();

    // Check Redis connectivity with PING
    let result: Result<String, _> = redis::cmd("PING").query_async(&mut conn).await;

    match result {
        Ok(response) if response == "PONG" => Ok((
            StatusCode::OK,
            Json(json!({
                "status": "ready",
                "checks": {
                    "redis": "ok"
                }
            })),
        )),
        Ok(response) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "status": "not_ready",
                "checks": {
                    "redis": format!("unexpected response: {}", response)
                }
            })),
        )),
        Err(e) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "status": "not_ready",
                "checks": {
                    "redis": format!("error: {}", e)
                }
            })),
        )),
    }
}

/// Stream info handler for monitoring.
///
/// Returns queue depth, entry IDs, and consumer group info.
pub async fn stream_info_handler(
    State(state): State<HealthState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut conn = (*state.redis).clone();

    // Use XINFO STREAM to get stream information
    let result: Result<redis::streams::StreamInfoStreamReply, _> = redis::cmd("XINFO")
        .arg("STREAM")
        .arg(&state.stream_name)
        .query_async(&mut conn)
        .await;

    match result {
        Ok(info) => Ok(Json(json!({
            "stream": state.stream_name,
            "length": info.length,
            "first_entry_id": info.first_entry.id,
            "last_entry_id": info.last_entry.id,
            "radix_tree_keys": info.radix_tree_keys,
            "groups": info.groups,
        }))),
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("no such key") || err_str.contains("ERR") {
                // Stream doesn't exist yet (no messages queued)
                Ok(Json(json!({
                    "stream": state.stream_name,
                    "length": 0,
                    "first_entry_id": null,
                    "last_entry_id": null,
                    "message": "Stream does not exist yet (no messages queued)"
                })))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": format!("Failed to get stream info: {}", e)
                    })),
                ))
            }
        }
    }
}

/// Prometheus metrics endpoint handler.
///
/// Returns metrics in Prometheus text format for scraping.
/// Use this for Prometheus `/metrics` endpoint.
pub async fn metrics_handler() -> impl IntoResponse {
    match metrics::get_metrics_handle() {
        Some(handle) => {
            let metrics_output = handle.render();
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                metrics_output,
            )
                .into_response()
        }
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            "Metrics not initialized. Call metrics::init_metrics() at startup.".to_string(),
        )
            .into_response(),
    }
}

// ============================================================================
// DLQ Admin Handlers
// ============================================================================

/// Query parameters for DLQ list endpoint.
#[derive(Debug, Deserialize)]
pub struct DlqListParams {
    /// Maximum number of messages to return (default: 10, max: 100)
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Number of messages to skip for pagination (default: 0)
    #[serde(default)]
    pub offset: usize,
}

fn default_limit() -> usize {
    10
}

/// Query parameters for DLQ reprocess batch endpoint.
#[derive(Debug, Deserialize)]
pub struct DlqReprocessParams {
    /// Number of messages to reprocess (default: 10, max: 100)
    #[serde(default = "default_limit")]
    pub count: usize,
}

/// Get DLQ statistics.
///
/// Returns the number of messages in the DLQ, oldest/newest message IDs.
///
/// `GET /admin/dlq/stats`
pub async fn dlq_stats_handler(
    State(state): State<HealthState>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let manager = state.dlq_manager();

    match manager.stats().await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

/// List messages in the DLQ.
///
/// Returns paginated list of DLQ messages with job data and error info.
///
/// `GET /admin/dlq/messages?limit=10&offset=0`
pub async fn dlq_list_handler(
    State(state): State<HealthState>,
    Query(params): Query<DlqListParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let manager = state.dlq_manager();

    // Cap limit at 100
    let limit = params.limit.min(100);

    match manager.list_messages(limit, params.offset).await {
        Ok(messages) => Ok(Json(json!({
            "messages": messages,
            "limit": limit,
            "offset": params.offset,
            "count": messages.len()
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

/// Reprocess a single message from the DLQ.
///
/// Moves the message back to the source stream for reprocessing.
///
/// `POST /admin/dlq/reprocess/:message_id`
pub async fn dlq_reprocess_one_handler(
    State(state): State<HealthState>,
    Path(message_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let manager = state.dlq_manager();

    match manager.reprocess_message(&message_id).await {
        Ok(true) => Ok((
            StatusCode::OK,
            Json(json!({
                "success": true,
                "message_id": message_id,
                "message": "Message requeued for processing"
            })),
        )),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Message not found in DLQ",
                "message_id": message_id
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

/// Reprocess a batch of messages from the DLQ.
///
/// Takes the oldest messages first and moves them back to the source stream.
///
/// `POST /admin/dlq/reprocess?count=10`
pub async fn dlq_reprocess_batch_handler(
    State(state): State<HealthState>,
    Query(params): Query<DlqReprocessParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let manager = state.dlq_manager();

    // Cap count at 100
    let count = params.count.min(100);

    match manager.reprocess_batch(count).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

/// Archive (delete) a single message from the DLQ.
///
/// Use this when a message should not be retried.
///
/// `DELETE /admin/dlq/:message_id`
pub async fn dlq_archive_one_handler(
    State(state): State<HealthState>,
    Path(message_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let manager = state.dlq_manager();

    match manager.archive_message(&message_id).await {
        Ok(true) => Ok((
            StatusCode::OK,
            Json(json!({
                "success": true,
                "message_id": message_id,
                "message": "Message archived (deleted from DLQ)"
            })),
        )),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Message not found in DLQ",
                "message_id": message_id
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

/// Archive all messages in the DLQ.
///
/// WARNING: This permanently deletes all DLQ messages.
///
/// `DELETE /admin/dlq/all`
pub async fn dlq_archive_all_handler(
    State(state): State<HealthState>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let manager = state.dlq_manager();

    match manager.archive_all().await {
        Ok(count) => Ok(Json(json!({
            "success": true,
            "archived_count": count,
            "message": "All DLQ messages archived"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

/// Create a standard health router.
///
/// This creates an Axum router with standard health endpoints:
/// - `/health` - Liveness probe
/// - `/healthz` - Liveness probe (K8s style)
/// - `/ready` - Readiness probe
/// - `/readyz` - Readiness probe (K8s style)
/// - `/stream/info` - Stream monitoring
/// - `/metrics` - Prometheus metrics
pub fn health_router(state: HealthState) -> axum::Router {
    use axum::routing::get;

    axum::Router::new()
        .route("/health", get(health_handler))
        .route("/healthz", get(health_handler))
        .route("/ready", get(ready_handler))
        .route("/readyz", get(ready_handler))
        .route("/stream/info", get(stream_info_handler))
        .route("/metrics", get(metrics_handler))
        .with_state(state)
}

/// Create a router with DLQ admin endpoints.
///
/// This creates an Axum router with DLQ management endpoints:
/// - `GET /admin/dlq/stats` - DLQ statistics
/// - `GET /admin/dlq/messages` - List DLQ messages
/// - `POST /admin/dlq/reprocess/:id` - Reprocess single message
/// - `POST /admin/dlq/reprocess` - Reprocess batch
/// - `DELETE /admin/dlq/:id` - Archive single message
/// - `DELETE /admin/dlq/all` - Archive all messages
pub fn dlq_admin_router(state: HealthState) -> axum::Router {
    use axum::routing::{delete, get, post};

    axum::Router::new()
        .route("/admin/dlq/stats", get(dlq_stats_handler))
        .route("/admin/dlq/messages", get(dlq_list_handler))
        .route("/admin/dlq/reprocess/{id}", post(dlq_reprocess_one_handler))
        .route("/admin/dlq/reprocess", post(dlq_reprocess_batch_handler))
        .route("/admin/dlq/{id}", delete(dlq_archive_one_handler))
        .route("/admin/dlq/all", delete(dlq_archive_all_handler))
        .with_state(state)
}

/// Create a full router with health and DLQ admin endpoints.
///
/// Combines health_router and dlq_admin_router.
pub fn full_admin_router(state: HealthState) -> axum::Router {
    use axum::routing::{delete, get, post};

    axum::Router::new()
        // Health endpoints
        .route("/health", get(health_handler))
        .route("/healthz", get(health_handler))
        .route("/ready", get(ready_handler))
        .route("/readyz", get(ready_handler))
        .route("/stream/info", get(stream_info_handler))
        .route("/metrics", get(metrics_handler))
        // DLQ admin endpoints
        .route("/admin/dlq/stats", get(dlq_stats_handler))
        .route("/admin/dlq/messages", get(dlq_list_handler))
        .route("/admin/dlq/reprocess/{id}", post(dlq_reprocess_one_handler))
        .route("/admin/dlq/reprocess", post(dlq_reprocess_batch_handler))
        .route("/admin/dlq/{id}", delete(dlq_archive_one_handler))
        .route("/admin/dlq/all", delete(dlq_archive_all_handler))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            status: "healthy",
            name: "test-worker".to_string(),
            version: "1.0.0".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"healthy\""));
        assert!(json.contains("\"name\":\"test-worker\""));
    }
}
