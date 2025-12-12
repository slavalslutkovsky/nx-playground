use axum::Router;
use domain_users::{handlers, PostgresUserRepository, UserService};

pub fn router(state: &crate::state::AppState) -> Router {
    // Use PostgreSQL repository with database connection
    let repository = PostgresUserRepository::new(state.db.clone());
    let service = UserService::new(repository.clone());

    // Return CRUD router (auth is now in separate /auth module)
    handlers::router(service)
}
