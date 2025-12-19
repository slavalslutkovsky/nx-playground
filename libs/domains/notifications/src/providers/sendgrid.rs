//! SendGrid email provider implementation.

use super::{EmailContent, EmailProvider, SentEmail};
use crate::error::{NotificationError, NotificationResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

/// SendGrid API configuration.
#[derive(Debug, Clone)]
pub struct SendGridConfig {
    /// SendGrid API key.
    pub api_key: String,
    /// Sender email address.
    pub from_email: String,
    /// Sender name.
    pub from_name: String,
    /// SendGrid API base URL (defaults to production).
    pub api_url: String,
}

impl SendGridConfig {
    /// Create a new SendGrid configuration.
    pub fn new(api_key: String, from_email: String, from_name: String) -> Self {
        Self {
            api_key,
            from_email,
            from_name,
            api_url: "https://api.sendgrid.com/v3".to_string(),
        }
    }

    /// Create configuration from environment variables.
    pub fn from_env() -> Result<Self, NotificationError> {
        let api_key = std::env::var("SENDGRID_API_KEY")
            .map_err(|_| NotificationError::ConfigError("SENDGRID_API_KEY not set".to_string()))?;
        let from_email = std::env::var("SENDGRID_FROM_EMAIL")
            .map_err(|_| NotificationError::ConfigError("SENDGRID_FROM_EMAIL not set".to_string()))?;
        let from_name = std::env::var("SENDGRID_FROM_NAME").unwrap_or_else(|_| "Zerg".to_string());

        Ok(Self::new(api_key, from_email, from_name))
    }
}

/// SendGrid email provider.
pub struct SendGridProvider {
    config: SendGridConfig,
    client: Client,
}

impl SendGridProvider {
    /// Create a new SendGrid provider.
    pub fn new(config: SendGridConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    /// Create a provider from environment variables.
    pub fn from_env() -> Result<Self, NotificationError> {
        let config = SendGridConfig::from_env()?;
        Ok(Self::new(config))
    }
}

// SendGrid API request/response structures

#[derive(Debug, Serialize)]
struct SendGridRequest {
    personalizations: Vec<Personalization>,
    from: EmailAddress,
    #[serde(skip_serializing_if = "Option::is_none")]
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

#[derive(Debug, Deserialize)]
struct SendGridError {
    errors: Vec<SendGridErrorDetail>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields are populated by deserialization from SendGrid API
struct SendGridErrorDetail {
    message: String,
    field: Option<String>,
}

#[async_trait]
impl EmailProvider for SendGridProvider {
    async fn send(&self, email: &EmailContent) -> NotificationResult<SentEmail> {
        // Build CC recipients
        let cc: Vec<EmailAddress> = email
            .cc
            .iter()
            .map(|addr| EmailAddress {
                email: addr.clone(),
                name: None,
            })
            .collect();

        // Build BCC recipients
        let bcc: Vec<EmailAddress> = email
            .bcc
            .iter()
            .map(|addr| EmailAddress {
                email: addr.clone(),
                name: None,
            })
            .collect();

        // Build reply-to if present
        let reply_to = email.reply_to.as_ref().map(|addr| EmailAddress {
            email: addr.clone(),
            name: None,
        });

        let request = SendGridRequest {
            personalizations: vec![Personalization {
                to: vec![EmailAddress {
                    email: email.to_email.clone(),
                    name: if email.to_name.is_empty() {
                        None
                    } else {
                        Some(email.to_name.clone())
                    },
                }],
                cc,
                bcc,
            }],
            from: EmailAddress {
                email: self.config.from_email.clone(),
                name: Some(self.config.from_name.clone()),
            },
            reply_to,
            subject: email.subject.clone(),
            content: vec![
                Content {
                    content_type: "text/plain".to_string(),
                    value: email.text_body.clone(),
                },
                Content {
                    content_type: "text/html".to_string(),
                    value: email.html_body.clone(),
                },
            ],
        };

        debug!(
            to = %email.to_email,
            subject = %email.subject,
            cc_count = email.cc.len(),
            bcc_count = email.bcc.len(),
            has_reply_to = email.reply_to.is_some(),
            "Sending email via SendGrid"
        );

        let response = self
            .client
            .post(format!("{}/mail/send", self.config.api_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        let message_id = response
            .headers()
            .get("x-message-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        if status.is_success() {
            info!(
                to = %email.to_email,
                message_id = ?message_id,
                "Email sent successfully via SendGrid"
            );
            Ok(SentEmail {
                message_id,
                accepted: true,
            })
        } else {
            let error_body = response.text().await.unwrap_or_default();
            error!(
                to = %email.to_email,
                status = %status,
                error = %error_body,
                "Failed to send email via SendGrid"
            );

            // Try to parse the error response
            let error_message = if let Ok(sg_error) = serde_json::from_str::<SendGridError>(&error_body) {
                sg_error
                    .errors
                    .into_iter()
                    .map(|e| e.message)
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                error_body
            };

            Err(NotificationError::ProviderError(format!(
                "SendGrid error ({}): {}",
                status, error_message
            )))
        }
    }

    fn name(&self) -> &'static str {
        "SendGrid"
    }

    async fn health_check(&self) -> NotificationResult<bool> {
        // SendGrid doesn't have a dedicated health endpoint,
        // so we check if the API key format is valid
        if self.config.api_key.starts_with("SG.") {
            Ok(true)
        } else {
            Err(NotificationError::ConfigError(
                "Invalid SendGrid API key format".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sendgrid_config_new() {
        let config = SendGridConfig::new(
            "SG.test_key".to_string(),
            "test@example.com".to_string(),
            "Test Sender".to_string(),
        );

        assert_eq!(config.api_key, "SG.test_key");
        assert_eq!(config.from_email, "test@example.com");
        assert_eq!(config.from_name, "Test Sender");
        assert_eq!(config.api_url, "https://api.sendgrid.com/v3");
    }
}
