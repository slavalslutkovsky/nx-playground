//! API routes module

pub mod health;
pub mod products;

use axum::Router;

use crate::state::AppState;

/// Create all API routes
pub fn routes(state: &AppState) -> Router {
    Router::new()
        .nest("/products", products::router(state))
        .merge(health::router(state.clone()))
}

/// Initialize database indexes
pub async fn init_indexes(state: &AppState) -> eyre::Result<()> {
    products::init_indexes(state).await
}
