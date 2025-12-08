use axum::Router;
use domain_tasks::{handlers, PgTaskRepository, TaskService};

pub fn router(state: &crate::state::AppState) -> Router {
    let repository = PgTaskRepository::new(state.db.clone());
    let service = TaskService::new(repository);
    handlers::direct_router(service)
}
