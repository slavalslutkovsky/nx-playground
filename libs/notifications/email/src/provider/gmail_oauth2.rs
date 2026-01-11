//! Gmail OAuth2 provider using refresh token
//!
//! Sends emails via Gmail API using OAuth2 with a pre-authorized refresh token.
//! This is simpler than Service Account setup and works with any Gmail account.
//!
//! ## Setup
//!
//! 1. Create OAuth2 credentials in Google Cloud Console:
//!    - Go to APIs & Services > Credentials
//!    - Create OAuth 2.0 Client ID (Desktop or Web application)
//!    - Note the Client ID and Client Secret
//!
//! 2. Enable Gmail API:
//!    - Go to APIs & Services > Library
//!    - Search for "Gmail API" and enable it
//!
//! 3. Get a refresh token (one-time setup):
//!    ```bash
//!    # Use OAuth2 playground or a script to get refresh token
//!    # Scope needed: https://www.googleapis.com/auth/gmail.send
//!    ```
//!
//! 4. Set environment variables:
//!    - `GMAIL_CLIENT_ID` - OAuth2 client ID
//!    - `GMAIL_CLIENT_SECRET` - OAuth2 client secret
//!    - `GMAIL_REFRESH_TOKEN` - Pre-authorized refresh token
//!    - `GMAIL_FROM_EMAIL` or `EMAIL_FROM_ADDRESS` - Sender email
//!    - `GMAIL_FROM_NAME` or `EMAIL_FROM_NAME` - Sender name (optional)

use crate::models::Email;
use crate::provider::{EmailProvider, SendResult};
use async_trait::async_trait;
use base64::Engine;
use eyre::{eyre, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error};

const GMAIL_API_URL: &str = "https://gmail.googleapis.com/gmail/v1/users/me/messages/send";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

/// Gmail OAuth2 provider using refresh token
pub struct GmailOAuth2Provider {
    client_id: String,
    client_secret: String,
    refresh_token: String,
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

/// Token response from Google OAuth2
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
    #[allow(dead_code)]
    token_type: String,
}

impl GmailOAuth2Provider {
    /// Create a new GmailOAuth2Provider
    pub fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        refresh_token: impl Into<String>,
        from_email: impl Into<String>,
        from_name: impl Into<String>,
    ) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            refresh_token: refresh_token.into(),
            from_email: from_email.into(),
            from_name: from_name.into(),
            client: Client::new(),
            token_cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Create from environment variables
    ///
    /// Expects:
    /// - `GMAIL_CLIENT_ID` - OAuth2 client ID
    /// - `GMAIL_CLIENT_SECRET` - OAuth2 client secret
    /// - `GMAIL_REFRESH_TOKEN` - Pre-authorized refresh token
    /// - `GMAIL_FROM_EMAIL` or `EMAIL_FROM_ADDRESS`
    /// - `GMAIL_FROM_NAME` or `EMAIL_FROM_NAME` (optional)
    pub fn from_env() -> Result<Self> {
        let client_id =
            std::env::var("GMAIL_CLIENT_ID").map_err(|_| eyre!("GMAIL_CLIENT_ID not set"))?;
        let client_secret = std::env::var("GMAIL_CLIENT_SECRET")
            .map_err(|_| eyre!("GMAIL_CLIENT_SECRET not set"))?;
        let refresh_token = std::env::var("GMAIL_REFRESH_TOKEN")
            .map_err(|_| eyre!("GMAIL_REFRESH_TOKEN not set"))?;

        let from_email = std::env::var("GMAIL_FROM_EMAIL")
            .or_else(|_| std::env::var("EMAIL_FROM_ADDRESS"))
            .map_err(|_| eyre!("GMAIL_FROM_EMAIL or EMAIL_FROM_ADDRESS not set"))?;

        let from_name = std::env::var("GMAIL_FROM_NAME")
            .or_else(|_| std::env::var("EMAIL_FROM_NAME"))
            .unwrap_or_else(|_| "Notifications".to_string());

        Ok(Self::new(
            client_id,
            client_secret,
            refresh_token,
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
        let token = self.refresh_access_token().await?;

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

    /// Refresh the access token using the refresh token
    async fn refresh_access_token(&self) -> Result<TokenResponse> {
        let response = self
            .client
            .post(TOKEN_URL)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("refresh_token", self.refresh_token.as_str()),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await
            .map_err(|e| eyre!("Token refresh request failed: {}", e))?;

        if !response.status().is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(eyre!("Token refresh failed: {}", error_body));
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
                message.push_str(&format!(
                    "Content-Type: multipart/alternative; boundary=\"{}\"\r\n\r\n",
                    boundary
                ));
                message.push_str(&format!("--{}\r\n", boundary));
                message.push_str("Content-Type: text/plain; charset=UTF-8\r\n\r\n");
                message.push_str(text);
                message.push_str("\r\n");
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
impl EmailProvider for GmailOAuth2Provider {
    async fn send(&self, email: &Email) -> Result<SendResult> {
        let access_token = self.get_access_token().await?;

        // Build raw email
        let raw_email = self.build_raw_email(email)?;

        // Base64url encode
        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw_email.as_bytes());

        let request = GmailSendRequest { raw: encoded };

        debug!(
            to = %email.to,
            subject = %email.subject,
            "Sending email via Gmail OAuth2"
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

            debug!(message_id = %gmail_response.id, "Email sent successfully via Gmail OAuth2");

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
        "gmail-oauth2"
    }
}
