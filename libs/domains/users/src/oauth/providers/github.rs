use crate::oauth::providers::{OAuthProvider, OAuthResult};
use crate::oauth::types::OAuthUserInfo;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct GithubProvider {
    client_id: String,
    client_secret: String,
    http_client: reqwest::Client,
}

#[derive(Debug, Serialize, Deserialize)]
struct GithubUserInfo {
    id: i64,
    login: String,
    email: Option<String>,
    name: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GithubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

impl GithubProvider {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
            http_client: reqwest::Client::new(),
        }
    }

    async fn fetch_primary_email(&self, access_token: &str) -> OAuthResult<Option<String>> {
        let response = self
            .http_client
            .get("https://api.github.com/user/emails")
            .bearer_auth(access_token)
            .header("User-Agent", "Zerg-OAuth-App")
            .send()
            .await
            .map_err(|e| {
                crate::error::UserError::OAuth(format!("Failed to get user emails: {}", e))
            })?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let emails: Vec<GithubEmail> = response.json().await.map_err(|e| {
            crate::error::UserError::OAuth(format!("Failed to parse emails: {}", e))
        })?;

        Ok(emails
            .into_iter()
            .find(|e| e.primary && e.verified)
            .map(|e| e.email))
    }
}

#[async_trait]
impl OAuthProvider for GithubProvider {
    fn name(&self) -> &str {
        "github"
    }

    fn required_scopes(&self) -> &'static [&'static str] {
        &["user:email", "read:user"]
    }

    fn auth_url(&self) -> &str {
        "https://github.com/login/oauth/authorize"
    }

    fn token_url(&self) -> &str {
        "https://github.com/login/oauth/access_token"
    }

    fn client_id(&self) -> &str {
        &self.client_id
    }

    fn client_secret(&self) -> &str {
        &self.client_secret
    }

    fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    async fn get_user_info(&self, access_token: &str) -> OAuthResult<OAuthUserInfo> {
        let response = self
            .http_client
            .get("https://api.github.com/user")
            .bearer_auth(access_token)
            .header("User-Agent", "Zerg-OAuth-App")
            .send()
            .await
            .map_err(|e| {
                crate::error::UserError::OAuth(format!("Failed to get user info: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(crate::error::UserError::OAuth(format!(
                "GitHub API returned error: {}",
                response.status()
            )));
        }

        let user_info: GithubUserInfo = response.json().await.map_err(|e| {
            crate::error::UserError::OAuth(format!("Failed to parse user info: {}", e))
        })?;

        let raw_data = serde_json::to_value(&user_info).map_err(|e| {
            crate::error::UserError::OAuth(format!("Failed to serialize user info: {}", e))
        })?;

        let email = if let Some(email) = user_info.email.clone() {
            Some(email)
        } else {
            self.fetch_primary_email(access_token).await?
        };

        let email_verified = email.is_some();

        Ok(OAuthUserInfo {
            provider_user_id: user_info.id.to_string(),
            email: email.clone(),
            email_verified,
            name: user_info.name.clone(),
            avatar_url: user_info.avatar_url.clone(),
            username: Some(user_info.login.clone()),
            raw_data,
        })
    }
}
