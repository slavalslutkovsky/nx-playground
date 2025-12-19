use axum::Router;
use domain_pricing::{handlers, PgPricingRepository, PricingService};

pub fn router(state: &crate::state::AppState) -> Router {
    let repository = PgPricingRepository::new(state.db.clone());
    let service = PricingService::new(repository);
    handlers::router(service)
}
