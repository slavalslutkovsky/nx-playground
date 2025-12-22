use axum::Router;

pub mod auth;
pub mod cloud_resources;
pub mod health;
pub mod projects;
pub mod tasks;
pub mod tasks_direct;
pub mod users;

/// Creates the API routes without the `/api` prefix.
/// The `/api` prefix will be added by the `create_router` helper.
///
/// This function takes a reference to AppState and initializes all services.
/// Returns a stateless Router (all sub-routers have state already applied).
/// Only Arc pointer clones remain when domains extract db connections (cheap).
///
/// Uses generated constants from SeaOrmResource proc macro to avoid hardcoded paths.
pub fn routes(state: &crate::state::AppState) -> Router {
    // Import ApiResource trait to access URL constants
    use domain_projects::ApiResource;

    Router::new()
        .nest("/auth", auth::router(state)) // Auth routes at /api/auth
        .nest("/tasks", tasks::router(state.clone()))
        .nest("/tasks-direct", tasks_direct::router(state))
        .nest(domain_projects::entity::Model::URL, projects::router(state))
        .nest(
            domain_cloud_resources::entity::Model::URL,
            cloud_resources::router(state),
        )
        .nest("/users", users::router(state)) // TODO: Add SeaOrmResource to domain_users
}

/// Creates a router with the /ready endpoint that performs actual health checks.
///
/// This router has state applied and can be merged with the stateless app router
/// from `create_router`. The /ready endpoint checks database and redis connections.
pub fn ready_router(state: crate::state::AppState) -> Router {
    use axum::routing::get;

    Router::new()
        .route("/ready", get(health::ready_handler))
        .with_state(state)
}
