use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{AppendHeaders, IntoResponse, Redirect, Response},
    routing::{get, post},
    Json, Router,
};
use axum_helpers::{JwtRedisAuth, ACCESS_TOKEN_TTL, REFRESH_TOKEN_TTL};
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::Deserialize;
use std::str::FromStr;

use crate::error::UserError;
use crate::models::{LoginRequest, LoginResponse, RegisterRequest};
use crate::repository::UserRepository;
use crate::service::UserService;

/// OAuth configuration
#[derive(Clone)]
pub struct OAuthConfig {
    pub google_client_id: String,
    pub google_client_secret: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub redirect_base_url: String,
    pub frontend_url: String,
}

/// Application state for auth handlers
#[derive(Clone)]
pub struct AuthState<R: UserRepository> {
    pub service: UserService<R>,
    pub oauth_config: OAuthConfig,
    pub jwt_auth: JwtRedisAuth,
}

/// Check if running in development mode
fn is_development() -> bool {
    std::env::var("APP_ENV")
        .map(|env| env == "development")
        .unwrap_or_else(|_| cfg!(debug_assertions))
}

/// Register a new user
async fn register<R: UserRepository>(
    State(state): State<AuthState<R>>,
    Json(input): Json<RegisterRequest>,
) -> Result<Response, UserError> {
    // Create user
    let user = state
        .service
        .create_user(crate::models::CreateUser {
            email: input.email,
            name: input.name,
            password: input.password,
            roles: vec![],
        })
        .await?;

    let user_id = user.id.to_string();

    // Create access token
    let access_token = state
        .jwt_auth
        .create_access_token(&user_id, &user.email, &user.name, &user.roles)
        .map_err(|e| {
            tracing::error!("Failed to create access token: {:?}", e);
            UserError::Internal("Failed to create token".to_string())
        })?;

    // Verify and whitelist access token
    let access_claims = state.jwt_auth.verify_token(&access_token).map_err(|e| {
        tracing::error!("Failed to verify access token: {:?}", e);
        UserError::Internal("Failed to verify token".to_string())
    })?;

    state
        .jwt_auth
        .whitelist_token(&access_claims.jti, &user_id, ACCESS_TOKEN_TTL as u64)
        .await
        .map_err(|e| {
            tracing::error!("Failed to whitelist access token: {:?}", e);
            UserError::Internal("Failed to whitelist token".to_string())
        })?;

    // Create refresh token
    let refresh_token = state
        .jwt_auth
        .create_refresh_token(&user_id, &user.email, &user.name, &user.roles)
        .map_err(|e| {
            tracing::error!("Failed to create refresh token: {:?}", e);
            UserError::Internal("Failed to create token".to_string())
        })?;

    // Verify and whitelist refresh token
    let refresh_claims = state.jwt_auth.verify_token(&refresh_token).map_err(|e| {
        tracing::error!("Failed to verify refresh token: {:?}", e);
        UserError::Internal("Failed to verify token".to_string())
    })?;

    state
        .jwt_auth
        .whitelist_token(
            &refresh_claims.jti,
            &user_id,
            REFRESH_TOKEN_TTL as u64,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to whitelist refresh token: {:?}", e);
            UserError::Internal("Failed to whitelist token".to_string())
        })?;

    // Create cookies
    let secure_flag = if is_development() { "" } else { " Secure;" };
    let access_cookie = format!(
        "access_token={}; HttpOnly;{} SameSite=Strict; Path=/; Max-Age={}",
        access_token, secure_flag, ACCESS_TOKEN_TTL
    );
    let refresh_cookie = format!(
        "refresh_token={}; HttpOnly;{} SameSite=Strict; Path=/; Max-Age={}",
        refresh_token, secure_flag, REFRESH_TOKEN_TTL
    );

    let response = LoginResponse { user };

    let access_cookie_header = HeaderValue::from_str(&access_cookie).map_err(|e| {
        UserError::Internal(format!("Failed to create cookie: {}", e))
    })?;
    let refresh_cookie_header = HeaderValue::from_str(&refresh_cookie).map_err(|e| {
        UserError::Internal(format!("Failed to create cookie: {}", e))
    })?;

    Ok((
        AppendHeaders([
            (header::SET_COOKIE, access_cookie_header),
            (header::SET_COOKIE, refresh_cookie_header),
        ]),
        Json(response),
    )
        .into_response())
}

/// Login with email and password
async fn login<R: UserRepository>(
    State(state): State<AuthState<R>>,
    Json(input): Json<LoginRequest>,
) -> Result<Response, UserError> {
    // Verify credentials
    let user = state
        .service
        .verify_credentials(&input.email, &input.password)
        .await?;

    let user_id = user.id.to_string();

    // Create access token
    let access_token = state
        .jwt_auth
        .create_access_token(&user_id, &user.email, &user.name, &user.roles)
        .map_err(|e| {
            tracing::error!("Failed to create access token: {:?}", e);
            UserError::Internal("Failed to create token".to_string())
        })?;

    // Verify and whitelist access token
    let access_claims = state.jwt_auth.verify_token(&access_token).map_err(|e| {
        tracing::error!("Failed to verify access token: {:?}", e);
        UserError::Internal("Failed to verify token".to_string())
    })?;

    state
        .jwt_auth
        .whitelist_token(&access_claims.jti, &user_id, ACCESS_TOKEN_TTL as u64)
        .await
        .map_err(|e| {
            tracing::error!("Failed to whitelist access token: {:?}", e);
            UserError::Internal("Failed to whitelist token".to_string())
        })?;

    // Create refresh token
    let refresh_token = state
        .jwt_auth
        .create_refresh_token(&user_id, &user.email, &user.name, &user.roles)
        .map_err(|e| {
            tracing::error!("Failed to create refresh token: {:?}", e);
            UserError::Internal("Failed to create token".to_string())
        })?;

    // Verify and whitelist refresh token
    let refresh_claims = state.jwt_auth.verify_token(&refresh_token).map_err(|e| {
        tracing::error!("Failed to verify refresh token: {:?}", e);
        UserError::Internal("Failed to verify token".to_string())
    })?;

    state
        .jwt_auth
        .whitelist_token(
            &refresh_claims.jti,
            &user_id,
            REFRESH_TOKEN_TTL as u64,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to whitelist refresh token: {:?}", e);
            UserError::Internal("Failed to whitelist token".to_string())
        })?;

    // Create cookies
    let secure_flag = if is_development() { "" } else { " Secure;" };
    let access_cookie = format!(
        "access_token={}; HttpOnly;{} SameSite=Strict; Path=/; Max-Age={}",
        access_token, secure_flag, ACCESS_TOKEN_TTL
    );
    let refresh_cookie = format!(
        "refresh_token={}; HttpOnly;{} SameSite=Strict; Path=/; Max-Age={}",
        refresh_token, secure_flag, REFRESH_TOKEN_TTL
    );

    let response = LoginResponse { user };

    let access_cookie_header = HeaderValue::from_str(&access_cookie).map_err(|e| {
        UserError::Internal(format!("Failed to create cookie: {}", e))
    })?;
    let refresh_cookie_header = HeaderValue::from_str(&refresh_cookie).map_err(|e| {
        UserError::Internal(format!("Failed to create cookie: {}", e))
    })?;

    Ok((
        AppendHeaders([
            (header::SET_COOKIE, access_cookie_header),
            (header::SET_COOKIE, refresh_cookie_header),
        ]),
        Json(response),
    )
        .into_response())
}

/// Logout
async fn logout<R: UserRepository>(
    State(state): State<AuthState<R>>,
    headers: axum::http::HeaderMap,
) -> Result<Response, UserError> {
    // Extract tokens from cookies
    let cookies_str = headers.get("cookie").and_then(|v| v.to_str().ok());

    if let Some(cookies) = cookies_str {
        // Revoke access token if present
        if let Some(access_token) = extract_cookie_value(cookies, "access_token") {
            if let Ok(claims) = state.jwt_auth.verify_token(&access_token) {
                let now = chrono::Utc::now().timestamp();
                let remaining_ttl = (claims.exp - now).max(0) as u64;

                let _ = state.jwt_auth.revoke_token(&claims.jti).await;
                let _ = state.jwt_auth.blacklist_token(&claims.jti, remaining_ttl).await;
                tracing::debug!("Revoked and blacklisted access token: {}", claims.jti);
            }
        }

        // Revoke refresh token if present
        if let Some(refresh_token) = extract_cookie_value(cookies, "refresh_token") {
            if let Ok(claims) = state.jwt_auth.verify_token(&refresh_token) {
                let now = chrono::Utc::now().timestamp();
                let remaining_ttl = (claims.exp - now).max(0) as u64;

                let _ = state.jwt_auth.revoke_token(&claims.jti).await;
                let _ = state.jwt_auth.blacklist_token(&claims.jti, remaining_ttl).await;
                tracing::debug!("Revoked and blacklisted refresh token: {}", claims.jti);
            }
        }
    }

    // Clear cookies
    let secure_flag = if is_development() { "" } else { " Secure;" };
    let clear_access = format!(
        "access_token=; HttpOnly;{} SameSite=Strict; Path=/; Max-Age=0",
        secure_flag
    );
    let clear_refresh = format!(
        "refresh_token=; HttpOnly;{} SameSite=Strict; Path=/; Max-Age=0",
        secure_flag
    );

    let clear_access_header = HeaderValue::from_str(&clear_access).map_err(|e| {
        UserError::Internal(format!("Failed to create cookie: {}", e))
    })?;
    let clear_refresh_header = HeaderValue::from_str(&clear_refresh).map_err(|e| {
        UserError::Internal(format!("Failed to create cookie: {}", e))
    })?;

    Ok((
        AppendHeaders([
            (header::SET_COOKIE, clear_access_header),
            (header::SET_COOKIE, clear_refresh_header),
        ]),
        StatusCode::NO_CONTENT,
    )
        .into_response())
}

/// Get current user from JWT claims
async fn me<R: UserRepository>(
    State(state): State<AuthState<R>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<crate::models::UserResponse>, UserError> {
    // Extract token from Authorization header or cookie
    let token = extract_token(&headers).ok_or(UserError::Unauthorized)?;

    // Verify token
    let claims = state
        .jwt_auth
        .verify_token(&token)
        .map_err(|_| UserError::Unauthorized)?;

    // Check not blacklisted
    if state
        .jwt_auth
        .is_token_blacklisted(&claims.jti)
        .await
        .map_err(|e| {
            tracing::error!("Redis error checking blacklist: {}", e);
            UserError::Internal("Service temporarily unavailable".to_string())
        })?
    {
        return Err(UserError::Unauthorized);
    }

    // Check whitelisted
    if !state
        .jwt_auth
        .is_token_whitelisted(&claims.jti)
        .await
        .map_err(|e| {
            tracing::error!("Redis error checking whitelist: {}", e);
            UserError::Internal("Service temporarily unavailable".to_string())
        })?
    {
        return Err(UserError::Unauthorized);
    }

    // Get full user from database
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| UserError::Unauthorized)?;

    let user = state.service.get_user(user_id).await?;

    Ok(Json(user))
}

/// Helper: Extract token from Authorization header or cookie
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer ").map(|s| s.to_string()))
        .or_else(|| extract_cookie_value(headers.get("cookie").and_then(|v| v.to_str().ok())?, "access_token"))
}

/// Helper: Extract cookie value by name
fn extract_cookie_value(cookies: &str, name: &str) -> Option<String> {
    cookies.split(';').find_map(|cookie| {
        let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
        if parts.len() == 2 && parts[0] == name {
            Some(parts[1].to_string())
        } else {
            None
        }
    })
}

/// Supported OAuth providers
#[derive(Debug, Clone, Copy)]
enum OAuthProvider {
    Google,
    Github,
}

impl FromStr for OAuthProvider {
    type Err = UserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "google" => Ok(OAuthProvider::Google),
            "github" => Ok(OAuthProvider::Github),
            _ => Err(UserError::OAuth(format!("Unsupported provider: {}", s))),
        }
    }
}

impl OAuthProvider {
    fn as_str(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "google",
            OAuthProvider::Github => "github",
        }
    }

    fn auth_url(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "https://accounts.google.com/o/oauth2/v2/auth",
            OAuthProvider::Github => "https://github.com/login/oauth/authorize",
        }
    }

    fn token_url(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "https://oauth2.googleapis.com/token",
            OAuthProvider::Github => "https://github.com/login/oauth/access_token",
        }
    }

    fn scopes(&self) -> Vec<Scope> {
        match self {
            OAuthProvider::Google => vec![
                Scope::new("email".to_string()),
                Scope::new("profile".to_string()),
            ],
            OAuthProvider::Github => vec![
                Scope::new("user:email".to_string()),
                Scope::new("read:user".to_string()),
            ],
        }
    }

    fn client_credentials(&self, config: &OAuthConfig) -> (String, String) {
        match self {
            OAuthProvider::Google => (
                config.google_client_id.clone(),
                config.google_client_secret.clone(),
            ),
            OAuthProvider::Github => (
                config.github_client_id.clone(),
                config.github_client_secret.clone(),
            ),
        }
    }
}

/// Query parameters for OAuth callback
#[derive(Debug, Deserialize)]
struct OAuthCallbackQuery {
    code: String,
    state: String,
}

/// Generate a secure random password that meets all validation requirements
fn generate_oauth_password() -> String {
    use rand::Rng;
    const CHARSET_LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    const CHARSET_UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const CHARSET_DIGIT: &[u8] = b"0123456789";
    const CHARSET_SPECIAL: &[u8] = b"!@#$%^&*()_+-=[]{}|;:,.<>?";

    let mut rng = rand::thread_rng();
    let mut password = String::new();

    // Ensure at least one of each required character type
    password.push(CHARSET_UPPER[rng.gen_range(0..CHARSET_UPPER.len())] as char);
    password.push(CHARSET_LOWER[rng.gen_range(0..CHARSET_LOWER.len())] as char);
    password.push(CHARSET_DIGIT[rng.gen_range(0..CHARSET_DIGIT.len())] as char);
    password.push(CHARSET_SPECIAL[rng.gen_range(0..CHARSET_SPECIAL.len())] as char);

    // Add remaining random characters (total 20 chars)
    let all_chars = [CHARSET_LOWER, CHARSET_UPPER, CHARSET_DIGIT, CHARSET_SPECIAL].concat();
    for _ in 0..16 {
        password.push(all_chars[rng.gen_range(0..all_chars.len())] as char);
    }

    // Shuffle to avoid predictable pattern
    let mut chars: Vec<char> = password.chars().collect();
    for i in (1..chars.len()).rev() {
        let j = rng.gen_range(0..=i);
        chars.swap(i, j);
    }

    chars.into_iter().collect()
}

/// Initiate OAuth flow for any provider
async fn authorize<R: UserRepository>(
    State(state): State<AuthState<R>>,
    Path(provider_str): Path<String>,
) -> Result<Redirect, UserError> {
    let provider = OAuthProvider::from_str(&provider_str)?;
    let redirect_url = format!(
        "{}/api/auth/oauth/{}/callback",
        state.oauth_config.redirect_base_url,
        provider.as_str()
    );
    tracing::info!("{} OAuth redirect URI: {}", provider_str, redirect_url);

    let (client_id, client_secret) = provider.client_credentials(&state.oauth_config);

    let client = BasicClient::new(ClientId::new(client_id))
        .set_client_secret(ClientSecret::new(client_secret))
        .set_auth_uri(
            AuthUrl::new(provider.auth_url().to_string())
                .map_err(|e| UserError::OAuth(format!("Invalid auth URL: {}", e)))?,
        )
        .set_token_uri(
            TokenUrl::new(provider.token_url().to_string())
                .map_err(|e| UserError::OAuth(format!("Invalid token URL: {}", e)))?,
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect_url)
                .map_err(|e| UserError::OAuth(format!("Invalid redirect URL: {}", e)))?,
        );

    let mut auth_request = client.authorize_url(CsrfToken::new_random);
    for scope in provider.scopes() {
        auth_request = auth_request.add_scope(scope);
    }
    let (auth_url, _csrf_token) = auth_request.url();

    Ok(Redirect::to(auth_url.as_str()))
}

/// Handle OAuth callback for any provider
async fn callback<R: UserRepository>(
    State(state): State<AuthState<R>>,
    Path(provider_str): Path<String>,
    Query(query): Query<OAuthCallbackQuery>,
) -> Result<Response, UserError> {
    let provider = OAuthProvider::from_str(&provider_str)?;
    let redirect_url = format!(
        "{}/api/auth/oauth/{}/callback",
        state.oauth_config.redirect_base_url,
        provider.as_str()
    );

    let (client_id, client_secret) = provider.client_credentials(&state.oauth_config);

    let client = BasicClient::new(ClientId::new(client_id))
        .set_client_secret(ClientSecret::new(client_secret))
        .set_auth_uri(
            AuthUrl::new(provider.auth_url().to_string())
                .map_err(|e| UserError::OAuth(format!("Invalid auth URL: {}", e)))?,
        )
        .set_token_uri(
            TokenUrl::new(provider.token_url().to_string())
                .map_err(|e| UserError::OAuth(format!("Invalid token URL: {}", e)))?,
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect_url)
                .map_err(|e| UserError::OAuth(format!("Invalid redirect URL: {}", e)))?,
        );

    // Exchange code for token
    let http_client = reqwest::Client::new();
    let token_result = client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(&http_client)
        .await
        .map_err(|e| UserError::OAuth(format!("Failed to exchange code: {}", e)))?;

    let access_token = token_result.access_token().secret();

    // Fetch user info from provider
    let user_info = match provider {
        OAuthProvider::Google => crate::oauth::fetch_google_user_info(access_token).await?,
        OAuthProvider::Github => crate::oauth::fetch_github_user_info(access_token).await?,
    };

    // Check if user exists by email
    let user = if let Ok(existing_user) = state.service.get_user_by_email(&user_info.email).await {
        // User exists, use it
        // TODO: Update user with Google OAuth ID if not already linked
        existing_user
    } else {
        // Create new user with OAuth info
        state
            .service
            .create_user(crate::models::CreateUser {
                email: user_info.email,
                name: user_info.name,
                password: generate_oauth_password(), // Secure random password for OAuth users
                roles: vec![],
            })
            .await?
    };

    let user_id = user.id.to_string();

    // Create JWT tokens
    let access_token = state
        .jwt_auth
        .create_access_token(&user_id, &user.email, &user.name, &user.roles)
        .map_err(|e| {
            tracing::error!("Failed to create access token: {:?}", e);
            UserError::Internal("Failed to create token".to_string())
        })?;

    let access_claims = state.jwt_auth.verify_token(&access_token).map_err(|e| {
        tracing::error!("Failed to verify access token: {:?}", e);
        UserError::Internal("Failed to verify token".to_string())
    })?;

    state
        .jwt_auth
        .whitelist_token(&access_claims.jti, &user_id, ACCESS_TOKEN_TTL as u64)
        .await
        .map_err(|e| {
            tracing::error!("Failed to whitelist access token: {:?}", e);
            UserError::Internal("Failed to whitelist token".to_string())
        })?;

    let refresh_token = state
        .jwt_auth
        .create_refresh_token(&user_id, &user.email, &user.name, &user.roles)
        .map_err(|e| {
            tracing::error!("Failed to create refresh token: {:?}", e);
            UserError::Internal("Failed to create token".to_string())
        })?;

    let refresh_claims = state.jwt_auth.verify_token(&refresh_token).map_err(|e| {
        tracing::error!("Failed to verify refresh token: {:?}", e);
        UserError::Internal("Failed to verify token".to_string())
    })?;

    state
        .jwt_auth
        .whitelist_token(
            &refresh_claims.jti,
            &user_id,
            REFRESH_TOKEN_TTL as u64,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to whitelist refresh token: {:?}", e);
            UserError::Internal("Failed to whitelist token".to_string())
        })?;

    // Create cookies for redirect
    let secure_flag = if is_development() { "" } else { " Secure;" };
    let access_cookie = format!(
        "access_token={}; HttpOnly;{} SameSite=Lax; Path=/; Max-Age={}",
        access_token, secure_flag, ACCESS_TOKEN_TTL
    );
    let refresh_cookie = format!(
        "refresh_token={}; HttpOnly;{} SameSite=Lax; Path=/; Max-Age={}",
        refresh_token, secure_flag, REFRESH_TOKEN_TTL
    );

    // Redirect to frontend with cookies set
    let redirect_url = format!("{}/tasks", state.oauth_config.frontend_url);

    let access_cookie_header = HeaderValue::from_str(&access_cookie).map_err(|e| {
        UserError::Internal(format!("Failed to create cookie: {}", e))
    })?;
    let refresh_cookie_header = HeaderValue::from_str(&refresh_cookie).map_err(|e| {
        UserError::Internal(format!("Failed to create cookie: {}", e))
    })?;

    Ok((
        AppendHeaders([
            (header::SET_COOKIE, access_cookie_header),
            (header::SET_COOKIE, refresh_cookie_header),
            (header::LOCATION, HeaderValue::from_str(&redirect_url).unwrap()),
        ]),
        StatusCode::FOUND,
    )
        .into_response())
}

/// Create auth router
pub fn auth_router<R>(state: AuthState<R>) -> Router
where
    R: UserRepository + Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/register", post(register::<R>))
        .route("/login", post(login::<R>))
        .route("/logout", post(logout::<R>))
        .route("/me", get(me::<R>))
        .route("/oauth/{provider}", get(authorize::<R>))
        .route("/oauth/{provider}/callback", get(callback::<R>))
        .with_state(state)
}
