use axum::Router;
use domain_tasks::handlers;

pub fn router(state: crate::state::AppState) -> Router {
    handlers::grpc_router(state.tasks_client.clone())
}
