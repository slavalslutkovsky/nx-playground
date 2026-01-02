//! API routes module
//!
//! This module defines all HTTP API routes for the MongoDB-based Zerg API.

pub mod events;
pub mod health;
pub mod items;

use axum::Router;

use crate::state::AppState;

/// Create all API routes
/// Note: These are nested under /api by axum_helpers::create_router
pub fn routes(state: &AppState) -> Router {
    Router::new()
        .nest("/events", events::router(state))
        .nest("/items", items::router(state))
        .merge(health::router(state.clone()))
}
