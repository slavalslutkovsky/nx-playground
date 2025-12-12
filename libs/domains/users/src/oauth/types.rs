use serde::{Deserialize, Serialize};

/// OAuth provider enumeration for identifying which provider
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Google,
    Github,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::Google => write!(f, "google"),
            Provider::Github => write!(f, "github"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub provider_user_id: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub username: Option<String>,
    pub raw_data: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub token_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthState {
    pub state: String,
    pub pkce_verifier: String,
    pub nonce: Option<String>,
    pub redirect_uri: String,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthCallbackParams {
    pub code: String,
    pub state: String,
}
