use axum::Router;
use domain_users::{
    AccountLinkingService, OAuthStateManager, PostgresOAuthAccountRepository,
    PostgresUserRepository, UserService,
    auth_handlers::{AuthState, OAuthConfig, auth_router},
};

pub fn router(state: &crate::state::AppState) -> Router {
    // Use PostgreSQL repository with database connection
    let user_repository = PostgresUserRepository::new(state.db.clone());
    let oauth_repository = PostgresOAuthAccountRepository::new(state.db.clone());
    let service = UserService::new(user_repository.clone());

    // Create OAuth configuration from app config
    let oauth_config = OAuthConfig {
        google_client_id: state.config.google_client_id.clone(),
        google_client_secret: state.config.google_client_secret.clone(),
        github_client_id: state.config.github_client_id.clone(),
        github_client_secret: state.config.github_client_secret.clone(),
        redirect_base_url: state.config.redirect_base_url.clone(),
        frontend_url: state.config.frontend_url.clone(),
    };

    // Create OAuth state manager for PKCE and CSRF protection
    let oauth_state_manager = OAuthStateManager::new(state.redis.clone());

    // Create account linking service
    let account_linking =
        AccountLinkingService::new(user_repository.clone(), oauth_repository.clone());

    // Create auth state with JWT authentication
    let auth_state = AuthState {
        service: service.clone(),
        oauth_config,
        jwt_auth: state.jwt_auth.clone(),
        oauth_state_manager,
        account_linking,
        notifications: Some(state.notifications.clone()),
    };

    // Return auth router
    auth_router(auth_state)
}
