use crate::oauth::providers::{OAuthProvider, OAuthResult};
use crate::oauth::types::OAuthUserInfo;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct GoogleProvider {
    client_id: String,
    client_secret: String,
    http_client: reqwest::Client,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleUserInfo {
    sub: String,
    email: Option<String>,
    email_verified: Option<bool>,
    name: Option<String>,
    picture: Option<String>,
}

impl GoogleProvider {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
            http_client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl OAuthProvider for GoogleProvider {
    fn name(&self) -> &str {
        "google"
    }

    fn required_scopes(&self) -> &'static [&'static str] {
        &["openid", "email", "profile"]
    }

    fn auth_url(&self) -> &str {
        "https://accounts.google.com/o/oauth2/v2/auth"
    }

    fn token_url(&self) -> &str {
        "https://oauth2.googleapis.com/token"
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
            .get("https://openidconnect.googleapis.com/v1/userinfo")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| {
                crate::error::UserError::OAuth(format!("Failed to get user info: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(crate::error::UserError::OAuth(format!(
                "Google API returned error: {}",
                response.status()
            )));
        }

        let user_info: GoogleUserInfo = response.json().await.map_err(|e| {
            crate::error::UserError::OAuth(format!("Failed to parse user info: {}", e))
        })?;

        let raw_data = serde_json::to_value(&user_info).map_err(|e| {
            crate::error::UserError::OAuth(format!("Failed to serialize user info: {}", e))
        })?;

        Ok(OAuthUserInfo {
            provider_user_id: user_info.sub,
            email: user_info.email.clone(),
            email_verified: user_info.email_verified.unwrap_or(false),
            name: user_info.name.clone(),
            avatar_url: user_info.picture.clone(),
            username: user_info.email,
            raw_data,
        })
    }
}
