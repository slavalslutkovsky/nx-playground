use axum::Router;
use domain_projects::{handlers, PgProjectRepository, ProjectService};

pub fn router(state: &crate::state::AppState) -> Router {
    let repository = PgProjectRepository::new(state.db.clone());
    let service = ProjectService::new(repository);
    handlers::router(service)
}
