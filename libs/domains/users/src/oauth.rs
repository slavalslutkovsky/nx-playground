// OAuth helper functions for Google and GitHub authentication
//
// Note: OAuth client creation is currently commented out due to oauth2 5.0 API changes.
// The client types change based on which builder methods are called, making it difficult
// to return a consistent `BasicClient` type.
//
// TODO: Implement OAuth handlers that create clients inline when needed,
// following the pattern from ~/private/playground

use serde::Deserialize;

use crate::error::{UserError, UserResult};
use crate::models::OAuthUserInfo;

// OAuth client creation helpers are commented out for now
// Will be implemented inline in OAuth handlers following playground pattern
/*
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken,
    RedirectUrl, Scope, TokenUrl,
};

pub fn create_google_oauth_client(...) -> BasicClient { ... }
pub fn create_github_oauth_client(...) -> BasicClient { ... }
pub fn generate_auth_url(...) -> (String, String) { ... }
*/

/// Google user info response structure
#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    id: String,
    email: String,
    name: String,
    picture: Option<String>,
}

/// GitHub user info response structure
#[derive(Debug, Deserialize)]
struct GitHubUserInfo {
    id: i64,
    login: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
}

/// GitHub email response structure
#[derive(Debug, Deserialize)]
struct GitHubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

/// Fetch Google user information
pub async fn fetch_google_user_info(access_token: &str) -> UserResult<OAuthUserInfo> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| UserError::OAuth(format!("Failed to fetch Google user info: {}", e)))?;

    if !response.status().is_success() {
        return Err(UserError::OAuth(format!(
            "Google API error: {}",
            response.status()
        )));
    }

    let google_user: GoogleUserInfo = response
        .json()
        .await
        .map_err(|e| UserError::OAuth(format!("Failed to parse Google user info: {}", e)))?;

    Ok(OAuthUserInfo {
        email: google_user.email,
        name: google_user.name,
        avatar_url: google_user.picture,
        provider_id: google_user.id,
    })
}

/// Fetch GitHub user information
pub async fn fetch_github_user_info(access_token: &str) -> UserResult<OAuthUserInfo> {
    let client = reqwest::Client::new();

    // Fetch user profile
    let user_response = client
        .get("https://api.github.com/user")
        .bearer_auth(access_token)
        .header("User-Agent", "zerg-api")
        .send()
        .await
        .map_err(|e| UserError::OAuth(format!("Failed to fetch GitHub user info: {}", e)))?;

    if !user_response.status().is_success() {
        return Err(UserError::OAuth(format!(
            "GitHub API error: {}",
            user_response.status()
        )));
    }

    let github_user: GitHubUserInfo = user_response
        .json()
        .await
        .map_err(|e| UserError::OAuth(format!("Failed to parse GitHub user info: {}", e)))?;

    // Fetch primary email if not in profile
    let email = if let Some(email) = github_user.email {
        email
    } else {
        // Fetch user emails
        let emails_response = client
            .get("https://api.github.com/user/emails")
            .bearer_auth(access_token)
            .header("User-Agent", "zerg-api")
            .send()
            .await
            .map_err(|e| UserError::OAuth(format!("Failed to fetch GitHub emails: {}", e)))?;

        let emails: Vec<GitHubEmail> = emails_response
            .json()
            .await
            .map_err(|e| UserError::OAuth(format!("Failed to parse GitHub emails: {}", e)))?;

        emails
            .into_iter()
            .find(|e| e.primary && e.verified)
            .map(|e| e.email)
            .ok_or_else(|| {
                UserError::OAuth("No verified primary email found in GitHub account".to_string())
            })?
    };

    Ok(OAuthUserInfo {
        email,
        name: github_user.name.unwrap_or(github_user.login),
        avatar_url: github_user.avatar_url,
        provider_id: github_user.id.to_string(),
    })
}
