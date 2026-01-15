//! Google Workspace Gmail API provider
//!
//! Sends emails via Gmail API using Service Account with domain-wide delegation.
//!
//! ## Setup Requirements
//!
//! 1. Create a Service Account in Google Cloud Console
//! 2. Enable domain-wide delegation for the service account
//! 3. In Google Workspace Admin, authorize the service account with scope:
//!    `https://www.googleapis.com/auth/gmail.send`
//! 4. Download the service account JSON key file
//!
//! ## Configuration
//!
//! Environment variables:
//! - `GOOGLE_SERVICE_ACCOUNT_KEY` - Base64-encoded service account JSON key, OR
//! - `GOOGLE_SERVICE_ACCOUNT_KEY_FILE` - Path to service account JSON key file
//! - `GMAIL_DELEGATED_USER` - Email of the user to impersonate (must be in your Workspace domain)
//! - `GMAIL_FROM_EMAIL` or `EMAIL_FROM_ADDRESS` - Default sender email
//! - `GMAIL_FROM_NAME` or `EMAIL_FROM_NAME` - Default sender name

use crate::models::Email;
use crate::provider::{EmailProvider, SendResult};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use eyre::{eyre, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error};

const GMAIL_API_URL: &str = "https://gmail.googleapis.com/gmail/v1/users/me/messages/send";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GMAIL_SEND_SCOPE: &str = "https://www.googleapis.com/auth/gmail.send";

/// Google Workspace Gmail API provider
pub struct GmailProvider {
    service_account: ServiceAccountKey,
    delegated_user: String,
    from_email: String,
    from_name: String,
    client: Client,
    /// Cached access token with expiry
    token_cache: Arc<RwLock<Option<CachedToken>>>,
}

#[derive(Clone)]
struct CachedToken {
    access_token: String,
    expires_at: u64,
}

/// Service account key structure (from Google JSON key file)
///
/// This matches the structure of the JSON key file downloaded from Google Cloud Console.
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceAccountKey {
    /// Service account email address
    pub client_email: String,
    /// RSA private key in PEM format
    pub private_key: String,
    /// Key ID (for reference)
    #[allow(dead_code)]
    pub private_key_id: String,
    /// OAuth2 token endpoint
    #[allow(dead_code)]
    pub token_uri: String,
}

/// JWT claims for Google OAuth2
#[derive(Debug, Serialize)]
struct JwtClaims {
    iss: String,   // Service account email
    sub: String,   // Delegated user email
    scope: String, // API scope
    aud: String,   // Token URL
    iat: u64,      // Issued at
    exp: u64,      // Expiry
}

/// Token response from Google OAuth2
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
    #[allow(dead_code)]
    token_type: String,
}

impl GmailProvider {
    /// Create a new GmailProvider
    pub fn new(
        service_account: ServiceAccountKey,
        delegated_user: impl Into<String>,
        from_email: impl Into<String>,
        from_name: impl Into<String>,
    ) -> Self {
        Self {
            service_account,
            delegated_user: delegated_user.into(),
            from_email: from_email.into(),
            from_name: from_name.into(),
            client: Client::new(),
            token_cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Create from environment variables
    ///
    /// Expects:
    /// - `GOOGLE_SERVICE_ACCOUNT_KEY` (base64-encoded JSON) OR `GOOGLE_SERVICE_ACCOUNT_KEY_FILE` (path)
    /// - `GMAIL_DELEGATED_USER` - User email to impersonate
    /// - `GMAIL_FROM_EMAIL` or `EMAIL_FROM_ADDRESS`
    /// - `GMAIL_FROM_NAME` or `EMAIL_FROM_NAME` (optional)
    pub fn from_env() -> Result<Self> {
        // Load service account key
        let service_account = if let Ok(key_base64) = std::env::var("GOOGLE_SERVICE_ACCOUNT_KEY") {
            let key_json = BASE64
                .decode(&key_base64)
                .map_err(|e| eyre!("Failed to decode GOOGLE_SERVICE_ACCOUNT_KEY: {}", e))?;
            serde_json::from_slice(&key_json)
                .map_err(|e| eyre!("Failed to parse service account key: {}", e))?
        } else if let Ok(key_path) = std::env::var("GOOGLE_SERVICE_ACCOUNT_KEY_FILE") {
            let key_json = std::fs::read_to_string(&key_path).map_err(|e| {
                eyre!(
                    "Failed to read service account key file {}: {}",
                    key_path,
                    e
                )
            })?;
            serde_json::from_str(&key_json)
                .map_err(|e| eyre!("Failed to parse service account key: {}", e))?
        } else {
            return Err(eyre!(
                "GOOGLE_SERVICE_ACCOUNT_KEY or GOOGLE_SERVICE_ACCOUNT_KEY_FILE must be set"
            ));
        };

        let delegated_user = std::env::var("GMAIL_DELEGATED_USER")
            .map_err(|_| eyre!("GMAIL_DELEGATED_USER not set"))?;

        let from_email = std::env::var("GMAIL_FROM_EMAIL")
            .or_else(|_| std::env::var("EMAIL_FROM_ADDRESS"))
            .map_err(|_| eyre!("GMAIL_FROM_EMAIL or EMAIL_FROM_ADDRESS not set"))?;

        let from_name = std::env::var("GMAIL_FROM_NAME")
            .or_else(|_| std::env::var("EMAIL_FROM_NAME"))
            .unwrap_or_else(|_| "Notifications".to_string());

        Ok(Self::new(
            service_account,
            delegated_user,
            from_email,
            from_name,
        ))
    }

    /// Get a valid access token, refreshing if necessary
    async fn get_access_token(&self) -> Result<String> {
        // Check cache first
        {
            let cache = self.token_cache.read().await;
            if let Some(cached) = cache.as_ref() {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                // Use token if it has at least 60 seconds before expiry
                if cached.expires_at > now + 60 {
                    return Ok(cached.access_token.clone());
                }
            }
        }

        // Need to refresh token
        let token = self.fetch_access_token().await?;

        // Cache the token
        {
            let mut cache = self.token_cache.write().await;
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            *cache = Some(CachedToken {
                access_token: token.access_token.clone(),
                expires_at: now + token.expires_in,
            });
        }

        Ok(token.access_token)
    }

    /// Fetch a new access token using JWT assertion
    async fn fetch_access_token(&self) -> Result<TokenResponse> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let claims = JwtClaims {
            iss: self.service_account.client_email.clone(),
            sub: self.delegated_user.clone(),
            scope: GMAIL_SEND_SCOPE.to_string(),
            aud: TOKEN_URL.to_string(),
            iat: now,
            exp: now + 3600, // 1 hour
        };

        // Create JWT
        let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
        let key =
            jsonwebtoken::EncodingKey::from_rsa_pem(self.service_account.private_key.as_bytes())
                .map_err(|e| eyre!("Invalid private key: {}", e))?;

        let jwt = jsonwebtoken::encode(&header, &claims, &key)
            .map_err(|e| eyre!("Failed to create JWT: {}", e))?;

        // Exchange JWT for access token
        let response = self
            .client
            .post(TOKEN_URL)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await
            .map_err(|e| eyre!("Token request failed: {}", e))?;

        if !response.status().is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(eyre!("Token exchange failed: {}", error_body));
        }

        response
            .json::<TokenResponse>()
            .await
            .map_err(|e| eyre!("Failed to parse token response: {}", e))
    }

    /// Build RFC 2822 email message
    fn build_raw_email(&self, email: &Email) -> Result<String> {
        let from_email = email.from.as_ref().unwrap_or(&self.from_email);
        let from = format!("{} <{}>", self.from_name, from_email);

        let boundary = format!("boundary_{}", uuid::Uuid::new_v4().simple());

        let mut message = String::new();

        // Headers
        message.push_str(&format!("From: {}\r\n", from));
        message.push_str(&format!("To: {}\r\n", email.to));

        if !email.cc.is_empty() {
            message.push_str(&format!("Cc: {}\r\n", email.cc.join(", ")));
        }

        if let Some(reply_to) = &email.reply_to {
            message.push_str(&format!("Reply-To: {}\r\n", reply_to));
        }

        message.push_str(&format!("Subject: {}\r\n", email.subject));
        message.push_str("MIME-Version: 1.0\r\n");

        // Build body based on content types
        match (&email.body_text, &email.body_html) {
            (Some(text), Some(html)) => {
                // Multipart alternative
                message.push_str(&format!(
                    "Content-Type: multipart/alternative; boundary=\"{}\"\r\n\r\n",
                    boundary
                ));

                // Plain text part
                message.push_str(&format!("--{}\r\n", boundary));
                message.push_str("Content-Type: text/plain; charset=UTF-8\r\n\r\n");
                message.push_str(text);
                message.push_str("\r\n");

                // HTML part
                message.push_str(&format!("--{}\r\n", boundary));
                message.push_str("Content-Type: text/html; charset=UTF-8\r\n\r\n");
                message.push_str(html);
                message.push_str("\r\n");

                message.push_str(&format!("--{}--\r\n", boundary));
            }
            (Some(text), None) => {
                message.push_str("Content-Type: text/plain; charset=UTF-8\r\n\r\n");
                message.push_str(text);
            }
            (None, Some(html)) => {
                message.push_str("Content-Type: text/html; charset=UTF-8\r\n\r\n");
                message.push_str(html);
            }
            (None, None) => {
                return Err(eyre!("Email must have text or HTML content"));
            }
        }

        Ok(message)
    }
}

/// Gmail API send request
#[derive(Debug, Serialize)]
struct GmailSendRequest {
    raw: String,
}

/// Gmail API send response
#[derive(Debug, Deserialize)]
struct GmailSendResponse {
    id: String,
    #[serde(rename = "threadId")]
    #[allow(dead_code)]
    thread_id: String,
}

#[async_trait]
impl EmailProvider for GmailProvider {
    async fn send(&self, email: &Email) -> Result<SendResult> {
        let access_token = self.get_access_token().await?;

        // Build raw email
        let raw_email = self.build_raw_email(email)?;

        // Base64url encode (Gmail API requires URL-safe base64)
        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw_email.as_bytes());

        let request = GmailSendRequest { raw: encoded };

        debug!(
            to = %email.to,
            subject = %email.subject,
            "Sending email via Gmail API"
        );

        let response = self
            .client
            .post(GMAIL_API_URL)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| eyre!("Gmail API request failed: {}", e))?;

        let status = response.status();

        if status.is_success() {
            let gmail_response: GmailSendResponse = response
                .json()
                .await
                .map_err(|e| eyre!("Failed to parse Gmail response: {}", e))?;

            debug!(message_id = %gmail_response.id, "Email sent successfully via Gmail");

            Ok(SendResult {
                message_id: gmail_response.id,
            })
        } else {
            let error_body = response.text().await.unwrap_or_default();
            error!(
                status = %status,
                error = %error_body,
                "Gmail API error"
            );

            // Map status codes to appropriate errors
            match status.as_u16() {
                429 => Err(eyre!("rate limit exceeded")),
                400 => Err(eyre!("invalid request: {}", error_body)),
                401 | 403 => Err(eyre!("authentication failed: {}", error_body)),
                _ => Err(eyre!("Gmail error ({}): {}", status, error_body)),
            }
        }
    }

    async fn health_check(&self) -> Result<()> {
        // Verify we can get an access token
        self.get_access_token().await?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "gmail"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_account_key_parsing() {
        let json = r#"{
            "type": "service_account",
            "client_email": "test@project.iam.gserviceaccount.com",
            "private_key": "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----\n",
            "private_key_id": "key123",
            "token_uri": "https://oauth2.googleapis.com/token"
        }"#;

        let key: ServiceAccountKey = serde_json::from_str(json).unwrap();
        assert_eq!(key.client_email, "test@project.iam.gserviceaccount.com");
    }
}
