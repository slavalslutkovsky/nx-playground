//! Health check endpoints

use axum::{routing::get, Json, Router};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
    version: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "products-api".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn ready(state: AppState) -> Json<HealthResponse> {
    // Check MongoDB connection
    let _ = state.db.list_collection_names().await;

    Json(HealthResponse {
        status: "ready".to_string(),
        service: "products-api".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(move || ready(state)))
}
