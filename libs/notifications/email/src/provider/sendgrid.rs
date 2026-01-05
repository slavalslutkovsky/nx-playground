//! SendGrid email provider
//!
//! Sends emails via SendGrid HTTP API.

use crate::models::Email;
use crate::provider::{EmailProvider, SendResult};
use async_trait::async_trait;
use eyre::{eyre, Result};
use reqwest::Client;
use serde::Serialize;
use tracing::{debug, error};

/// SendGrid API endpoint
const SENDGRID_API_URL: &str = "https://api.sendgrid.com/v3/mail/send";

/// SendGrid email provider
pub struct SendGridProvider {
    api_key: String,
    from_email: String,
    from_name: String,
    client: Client,
}

impl SendGridProvider {
    /// Create a new SendGridProvider
    pub fn new(
        api_key: impl Into<String>,
        from_email: impl Into<String>,
        from_name: impl Into<String>,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            from_email: from_email.into(),
            from_name: from_name.into(),
            client: Client::new(),
        }
    }

    /// Create from environment variables
    ///
    /// Expects:
    /// - `SENDGRID_API_KEY`
    /// - `SENDGRID_FROM_EMAIL` or `EMAIL_FROM_ADDRESS`
    /// - `SENDGRID_FROM_NAME` or `EMAIL_FROM_NAME`
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("SENDGRID_API_KEY")
            .map_err(|_| eyre!("SENDGRID_API_KEY not set"))?;

        let from_email = std::env::var("SENDGRID_FROM_EMAIL")
            .or_else(|_| std::env::var("EMAIL_FROM_ADDRESS"))
            .map_err(|_| eyre!("SENDGRID_FROM_EMAIL or EMAIL_FROM_ADDRESS not set"))?;

        let from_name = std::env::var("SENDGRID_FROM_NAME")
            .or_else(|_| std::env::var("EMAIL_FROM_NAME"))
            .unwrap_or_else(|_| "Notifications".to_string());

        Ok(Self::new(api_key, from_email, from_name))
    }
}

/// SendGrid API request payload
#[derive(Debug, Serialize)]
struct SendGridRequest {
    personalizations: Vec<Personalization>,
    from: EmailAddress,
    reply_to: Option<EmailAddress>,
    subject: String,
    content: Vec<Content>,
}

#[derive(Debug, Serialize)]
struct Personalization {
    to: Vec<EmailAddress>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    cc: Vec<EmailAddress>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    bcc: Vec<EmailAddress>,
}

#[derive(Debug, Serialize)]
struct EmailAddress {
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Debug, Serialize)]
struct Content {
    #[serde(rename = "type")]
    content_type: String,
    value: String,
}

#[async_trait]
impl EmailProvider for SendGridProvider {
    async fn send(&self, email: &Email) -> Result<SendResult> {
        // Build content
        let mut content = Vec::new();

        if let Some(text) = &email.body_text {
            content.push(Content {
                content_type: "text/plain".to_string(),
                value: text.clone(),
            });
        }

        if let Some(html) = &email.body_html {
            content.push(Content {
                content_type: "text/html".to_string(),
                value: html.clone(),
            });
        }

        if content.is_empty() {
            return Err(eyre!("Email must have text or HTML content"));
        }

        // Build personalization
        let mut personalization = Personalization {
            to: vec![EmailAddress {
                email: email.to.clone(),
                name: None,
            }],
            cc: Vec::new(),
            bcc: Vec::new(),
        };

        // Add CC
        for cc in &email.cc {
            personalization.cc.push(EmailAddress {
                email: cc.clone(),
                name: None,
            });
        }

        // Add BCC
        for bcc in &email.bcc {
            personalization.bcc.push(EmailAddress {
                email: bcc.clone(),
                name: None,
            });
        }

        // Build request
        let request = SendGridRequest {
            personalizations: vec![personalization],
            from: EmailAddress {
                email: email
                    .from
                    .clone()
                    .unwrap_or_else(|| self.from_email.clone()),
                name: Some(self.from_name.clone()),
            },
            reply_to: email.reply_to.as_ref().map(|r| EmailAddress {
                email: r.clone(),
                name: None,
            }),
            subject: email.subject.clone(),
            content,
        };

        debug!(
            to = %email.to,
            subject = %email.subject,
            "Sending email via SendGrid"
        );

        // Send request
        let response = self
            .client
            .post(SENDGRID_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| eyre!("SendGrid request failed: {}", e))?;

        let status = response.status();

        if status.is_success() {
            // SendGrid returns message ID in X-Message-Id header
            let message_id = response
                .headers()
                .get("X-Message-Id")
                .and_then(|v| v.to_str().ok())
                .unwrap_or(&email.id)
                .to_string();

            debug!(message_id = %message_id, "Email sent successfully");

            Ok(SendResult { message_id })
        } else {
            let error_body = response.text().await.unwrap_or_default();
            error!(
                status = %status,
                error = %error_body,
                "SendGrid API error"
            );

            // Map status codes to appropriate errors
            match status.as_u16() {
                429 => Err(eyre!("rate limit exceeded")),
                400 => Err(eyre!("invalid request: {}", error_body)),
                401 | 403 => Err(eyre!("authentication failed")),
                _ => Err(eyre!("SendGrid error ({}): {}", status, error_body)),
            }
        }
    }

    async fn health_check(&self) -> Result<()> {
        // Simple validation that API key is set
        if self.api_key.is_empty() {
            return Err(eyre!("SendGrid API key not configured"));
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "sendgrid"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_address_serialization() {
        let addr = EmailAddress {
            email: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
        };

        let json = serde_json::to_string(&addr).unwrap();
        assert!(json.contains("test@example.com"));
        assert!(json.contains("Test User"));
    }
}
