//! WorkOS AuthKit OAuth provider implementation
//!
//! WorkOS AuthKit provides a hosted authentication UI that handles:
//! - Email/password authentication
//! - Social logins (Google, GitHub, etc.)
//! - Enterprise SSO (SAML, OIDC)
//! - MFA
//!
//! The flow is similar to OAuth2 but uses WorkOS-specific endpoints.

use crate::error::UserError;
use crate::oauth::providers::{OAuthProvider, OAuthResult};
use crate::oauth::types::{OAuthUserInfo, TokenResponse};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// WorkOS AuthKit provider
#[derive(Clone)]
pub struct WorkOsProvider {
    client_id: String,
    api_key: String,
    #[allow(dead_code)] // Stored for potential future use
    redirect_uri: String,
    http_client: reqwest::Client,
}

/// WorkOS user object returned from authentication
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkOsUser {
    pub id: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    #[serde(default)]
    pub email_verified: bool,
    pub profile_picture_url: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl Default for WorkOsUser {
    fn default() -> Self {
        Self {
            id: String::new(),
            email: String::new(),
            first_name: None,
            last_name: None,
            email_verified: false,
            profile_picture_url: None,
            created_at: None,
            updated_at: None,
        }
    }
}

/// OAuth tokens from upstream provider (Google, GitHub, etc.)
/// Only returned when "Return OAuth tokens" is enabled in WorkOS dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamOAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub scopes: Option<Vec<String>>,
}

/// WorkOS authentication response
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkOsAuthResponse {
    pub user: WorkOsUser,
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    /// OAuth tokens from upstream provider (Google, GitHub, etc.)
    /// Only present when "Return OAuth tokens" is enabled in WorkOS dashboard
    #[serde(default)]
    pub oauth_tokens: Option<UpstreamOAuthTokens>,
    /// Authentication method - a string like "GitHubOAuth", "GoogleOAuth", "Password", etc.
    #[serde(default)]
    pub authentication_method: Option<String>,
}

/// WorkOS authentication request body
#[derive(Debug, Serialize)]
struct WorkOsAuthRequest {
    client_id: String,
    client_secret: String,
    grant_type: String,
    code: String,
}

impl WorkOsProvider {
    pub fn new(client_id: String, api_key: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            api_key,
            redirect_uri,
            http_client: reqwest::Client::new(),
        }
    }

    /// Exchange authorization code for user and tokens
    /// This is WorkOS-specific and returns user data directly
    pub async fn authenticate(&self, code: &str) -> Result<WorkOsAuthResponse, UserError> {
        let request_body = WorkOsAuthRequest {
            client_id: self.client_id.clone(),
            client_secret: self.api_key.clone(),
            grant_type: "authorization_code".to_string(),
            code: code.to_string(),
        };

        let response = self
            .http_client
            .post("https://api.workos.com/user_management/authenticate")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| UserError::OAuth(format!("Failed to authenticate with WorkOS: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(UserError::OAuth(format!(
                "WorkOS authentication failed: {} - {}",
                status, body
            )));
        }

        // Get raw response text for debugging
        let response_text = response
            .text()
            .await
            .map_err(|e| UserError::OAuth(format!("Failed to read WorkOS response: {}", e)))?;

        tracing::debug!("WorkOS raw response: {}", response_text);

        // Parse the response
        serde_json::from_str::<WorkOsAuthResponse>(&response_text).map_err(|e| {
            tracing::error!(
                "Failed to parse WorkOS response: {}. Raw response: {}",
                e,
                response_text
            );
            UserError::OAuth(format!("Failed to parse WorkOS response: {}", e))
        })
    }
}

#[async_trait]
impl OAuthProvider for WorkOsProvider {
    fn name(&self) -> &str {
        "workos"
    }

    fn required_scopes(&self) -> &'static [&'static str] {
        // WorkOS AuthKit handles scopes internally based on dashboard configuration
        // Google/GitHub scopes are configured in WorkOS dashboard, not here
        &[]
    }

    fn auth_url(&self) -> &str {
        "https://api.workos.com/user_management/authorize"
    }

    fn token_url(&self) -> &str {
        "https://api.workos.com/user_management/authenticate"
    }

    fn client_id(&self) -> &str {
        &self.client_id
    }

    fn client_secret(&self) -> &str {
        &self.api_key
    }

    fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    /// Override authorize_url to use WorkOS-specific parameters
    fn authorize_url(
        &self,
        state: &str,
        _pkce_verifier_str: &str,
        redirect_uri: &str,
        _nonce: Option<&str>,
    ) -> Result<String, UserError> {
        // WorkOS AuthKit uses different parameters than standard OAuth2
        // provider=authkit tells WorkOS to use the AuthKit hosted UI
        let url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&provider=authkit&state={}",
            self.auth_url(),
            urlencoding::encode(&self.client_id),
            urlencoding::encode(redirect_uri),
            urlencoding::encode(state)
        );

        Ok(url)
    }

    /// Override exchange_code since WorkOS API differs from standard OAuth2
    async fn exchange_code(
        &self,
        code: &str,
        _pkce_verifier: &str,
        _redirect_uri: &str,
    ) -> Result<TokenResponse, UserError> {
        // WorkOS authenticate endpoint returns user + tokens together
        // We call authenticate() and extract the token part
        let auth_response = self.authenticate(code).await?;

        Ok(TokenResponse {
            access_token: auth_response.access_token,
            refresh_token: auth_response.refresh_token,
            expires_in: None, // WorkOS doesn't return this in authenticate response
            token_type: "Bearer".to_string(),
        })
    }

    /// Get user info - for WorkOS, this data comes from the authenticate response
    /// This method is called after exchange_code, so we need to store the user data
    /// In practice, the auth handler should use authenticate() directly to get both
    async fn get_user_info(&self, _access_token: &str) -> OAuthResult<OAuthUserInfo> {
        // For WorkOS, user info comes directly from the authenticate response
        // The access token can be used to call other WorkOS APIs if needed
        // For now, return an error since we expect the handler to use authenticate() directly
        Err(UserError::OAuth(
            "Use WorkOsProvider::authenticate() to get user info directly".to_string(),
        ))
    }
}

/// Helper function to convert WorkOS user to OAuthUserInfo
pub fn workos_user_to_oauth_info(user: &WorkOsUser) -> OAuthUserInfo {
    let name = match (&user.first_name, &user.last_name) {
        (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
        (Some(first), None) => Some(first.clone()),
        (None, Some(last)) => Some(last.clone()),
        (None, None) => None,
    };

    let raw_data = serde_json::to_value(user).unwrap_or(serde_json::Value::Null);

    OAuthUserInfo {
        provider_user_id: user.id.clone(),
        email: Some(user.email.clone()),
        email_verified: user.email_verified,
        name,
        avatar_url: user.profile_picture_url.clone(),
        username: Some(user.email.clone()),
        raw_data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authorize_url_generation() {
        let provider = WorkOsProvider::new(
            "client_123".to_string(),
            "sk_test_456".to_string(),
            "http://localhost:8080/callback".to_string(),
        );

        let url = provider
            .authorize_url("state123", "pkce_verifier", "http://localhost:8080/callback", None)
            .unwrap();

        assert!(url.contains("client_id=client_123"));
        assert!(url.contains("provider=authkit"));
        assert!(url.contains("state=state123"));
        assert!(url.contains("response_type=code"));
    }

    #[test]
    fn test_workos_user_to_oauth_info() {
        let user = WorkOsUser {
            id: "user_123".to_string(),
            email: "test@example.com".to_string(),
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
            email_verified: true,
            profile_picture_url: Some("https://example.com/avatar.jpg".to_string()),
            created_at: None,
            updated_at: None,
        };

        let oauth_info = workos_user_to_oauth_info(&user);

        assert_eq!(oauth_info.provider_user_id, "user_123");
        assert_eq!(oauth_info.email, Some("test@example.com".to_string()));
        assert_eq!(oauth_info.name, Some("John Doe".to_string()));
        assert!(oauth_info.email_verified);
    }
}
