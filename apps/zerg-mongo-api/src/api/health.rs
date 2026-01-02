//! Health check endpoints

use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    mongodb: bool,
}

/// Create a health check router
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/ready", get(readiness_check))
        .with_state(state)
}

/// Readiness check - verifies MongoDB connection
async fn readiness_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let mongodb_healthy = database::mongodb::check_health(&state.mongo_client).await;

    Json(HealthResponse {
        status: if mongodb_healthy {
            "ready"
        } else {
            "unhealthy"
        }
        .to_string(),
        mongodb: mongodb_healthy,
    })
}
