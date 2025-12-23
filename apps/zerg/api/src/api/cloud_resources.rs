use axum::Router;
use domain_cloud_resources::{CloudResourceService, PgCloudResourceRepository, handlers};

pub fn router(state: &crate::state::AppState) -> Router {
    let repository = PgCloudResourceRepository::new(state.db.clone());
    let service = CloudResourceService::new(repository);
    handlers::router(service)
}
