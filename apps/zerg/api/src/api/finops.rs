//! FinOps AI Agent API routes
//!
//! Provides chat-based FinOps optimization assistance with streaming responses.

use axum::Router;
use domain_finops::{
    FinopsOrchestrator, FinopsService, FinopsState, PgFinopsRepository,
    handlers,
};
use domain_pricing::{PgPricingRepository, PricingService};
use std::sync::Arc;

pub fn router(state: &crate::state::AppState) -> Router {
    // Create finops repository and service
    let finops_repository = PgFinopsRepository::new(state.db.clone());
    let finops_service = FinopsService::new(finops_repository.clone());

    // Create pricing service for the orchestrator's price comparison tool
    let pricing_repository = PgPricingRepository::new(state.db.clone());
    let pricing_service = Arc::new(PricingService::new(pricing_repository));

    // Create the AI orchestrator with pricing capabilities
    let orchestrator = FinopsOrchestrator::with_pricing_service(
        FinopsService::new(finops_repository),
        pricing_service,
    );

    // Create the shared state for handlers
    let finops_state = FinopsState::new(finops_service, orchestrator);

    handlers::router(finops_state)
}
