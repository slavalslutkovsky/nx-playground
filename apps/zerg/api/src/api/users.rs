use axum::Router;
use domain_users::{handlers, InMemoryUserRepository, UserService};

pub fn router(_state: &crate::AppState) -> Router {
    let repository = InMemoryUserRepository::new();
    let service = UserService::new(repository);
    handlers::router(service)
}
