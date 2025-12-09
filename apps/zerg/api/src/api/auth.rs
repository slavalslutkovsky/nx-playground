use axum::Router;
use domain_users::{
    auth_handlers::{auth_router, AuthState, OAuthConfig},
    PostgresUserRepository,
    UserService,
};

pub fn router(state: &crate::state::AppState) -> Router {
    // Use PostgreSQL repository with database connection
    let repository = PostgresUserRepository::new(state.db.clone());
    let service = UserService::new(repository.clone());

    // Create OAuth configuration from app config
    let oauth_config = OAuthConfig {
        google_client_id: state.config.google_client_id.clone(),
        google_client_secret: state.config.google_client_secret.clone(),
        github_client_id: state.config.github_client_id.clone(),
        github_client_secret: state.config.github_client_secret.clone(),
        redirect_base_url: state.config.redirect_base_url.clone(),
        frontend_url: state.config.frontend_url.clone(),
    };

    // Create auth state with JWT authentication
    let auth_state = AuthState {
        service: service.clone(),
        oauth_config,
        jwt_auth: state.jwt_auth.clone(),
    };

    // Return auth router
    auth_router(auth_state)
}
