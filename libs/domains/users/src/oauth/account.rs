use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// OAuth account linked to a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub provider_username: Option<String>,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub scopes: Option<Vec<String>>,
    pub raw_user_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Parameters for creating a new OAuth account
#[derive(Debug, Clone)]
pub struct CreateOAuthAccountParams<'a> {
    pub user_id: Uuid,
    pub provider: &'a str,
    pub provider_user_id: &'a str,
    pub provider_username: Option<&'a str>,
    pub email: Option<&'a str>,
    pub display_name: Option<&'a str>,
    pub avatar_url: Option<&'a str>,
    pub access_token: Option<&'a str>,
    pub refresh_token: Option<&'a str>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub scopes: Option<Vec<String>>,
    pub raw_user_data: Option<serde_json::Value>,
}
