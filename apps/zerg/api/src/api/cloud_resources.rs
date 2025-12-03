use axum::Router;
use domain_cloud_resources::{handlers, CloudResourceService, PgCloudResourceRepository};

pub fn router(state: &crate::AppState) -> Router {
    let repository = PgCloudResourceRepository::new(state.db.clone());
    let service = CloudResourceService::new(repository);
    handlers::router(service)
}
