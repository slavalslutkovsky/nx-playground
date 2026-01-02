//! Products API routes

use axum::Router;
use domain_products::{handlers, MongoProductRepository, ProductService};

use crate::state::AppState;

/// Create products router
pub fn router(state: &AppState) -> Router {
    let repository = MongoProductRepository::new(&state.db);
    let service = ProductService::new(repository);
    handlers::router(service)
}

/// Initialize products indexes
pub async fn init_indexes(state: &AppState) -> eyre::Result<()> {
    let repository = MongoProductRepository::new(&state.db);
    repository.init_indexes().await?;
    Ok(())
}
