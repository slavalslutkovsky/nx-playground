//! Items API routes
//!
//! This module wires up the items domain to HTTP routes.

use axum::Router;
use domain_items::{ItemService, MongoItemRepository, handlers};

use crate::state::AppState;

/// Create items router
pub fn router(state: &AppState) -> Router {
    // Create the MongoDB repository
    let repository = MongoItemRepository::new(state.db.clone());

    // Create the service
    let service = ItemService::new(repository);

    // Return the domain's router
    handlers::router(service)
}
