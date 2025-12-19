//! SMTP email provider implementation using lettre.
//!
//! This provider is primarily intended for local development with MailHog/Mailpit
//! or similar SMTP testing tools.

use super::{EmailContent, EmailProvider, SentEmail};
use crate::error::{NotificationError, NotificationResult};
use async_trait::async_trait;
use lettre::{
    message::{header::ContentType, Mailbox, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use std::sync::Arc;
use tracing::{debug, error, info};

/// SMTP configuration.
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    /// SMTP server host.
    pub host: String,
    /// SMTP server port.
    pub port: u16,
    /// Sender email address.
    pub from_email: String,
    /// Sender name.
    pub from_name: String,
    /// SMTP username (optional for dev servers like Mailpit).
    pub username: Option<String>,
    /// SMTP password (optional for dev servers like Mailpit).
    pub password: Option<String>,
    /// Whether to use TLS (false for local dev servers).
    pub use_tls: bool,
}

impl SmtpConfig {
    /// Create a new SMTP configuration.
    pub fn new(host: String, port: u16, from_email: String, from_name: String) -> Self {
        Self {
            host,
            port,
            from_email,
            from_name,
            username: None,
            password: None,
            use_tls: false,
        }
    }

    /// Create configuration for MailHog/Mailpit (default development setup).
    pub fn mailhog() -> Self {
        Self {
            host: std::env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("SMTP_PORT")
                .unwrap_or_else(|_| "1025".to_string())
                .parse()
                .unwrap_or(1025),
            from_email: std::env::var("SMTP_FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@localhost".to_string()),
            from_name: std::env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "Zerg Dev".to_string()),
            username: std::env::var("SMTP_USERNAME").ok(),
            password: std::env::var("SMTP_PASSWORD").ok(),
            use_tls: std::env::var("SMTP_USE_TLS")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
        }
    }

    /// Builder method to set TLS.
    pub fn with_tls(mut self, use_tls: bool) -> Self {
        self.use_tls = use_tls;
        self
    }

    /// Builder method to set credentials.
    pub fn with_credentials(mut self, username: String, password: String) -> Self {
        self.username = Some(username);
        self.password = Some(password);
        self
    }
}

/// SMTP email provider for development.
pub struct SmtpProvider {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    config: Arc<SmtpConfig>,
}

impl SmtpProvider {
    /// Create a new SMTP provider.
    pub fn new(config: SmtpConfig) -> NotificationResult<Self> {
        let transport = Self::build_transport(&config)?;
        Ok(Self {
            transport,
            config: Arc::new(config),
        })
    }

    /// Create a provider configured for MailHog/Mailpit.
    pub fn mailhog() -> NotificationResult<Self> {
        Self::new(SmtpConfig::mailhog())
    }

    /// Build the SMTP transport based on configuration.
    fn build_transport(config: &SmtpConfig) -> NotificationResult<AsyncSmtpTransport<Tokio1Executor>> {
        let transport = if config.use_tls {
            // TLS-enabled transport (for production SMTP servers)
            let mut builder = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
                .map_err(|e| NotificationError::ProviderError(format!("Failed to create SMTP relay: {}", e)))?
                .port(config.port);

            // Add credentials if provided
            if let (Some(username), Some(password)) = (&config.username, &config.password) {
                builder = builder.credentials(Credentials::new(username.clone(), password.clone()));
            }

            builder.build()
        } else {
            // Non-TLS transport (for local dev servers like Mailpit)
            let mut builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
                .port(config.port);

            // Add credentials if provided (some dev servers might require them)
            if let (Some(username), Some(password)) = (&config.username, &config.password) {
                builder = builder.credentials(Credentials::new(username.clone(), password.clone()));
            }

            builder.build()
        };

        Ok(transport)
    }

    /// Build a lettre Message from EmailContent.
    fn build_message(&self, email: &EmailContent) -> NotificationResult<Message> {
        let from: Mailbox = format!("{} <{}>", self.config.from_name, self.config.from_email)
            .parse()
            .map_err(|e| NotificationError::ProviderError(format!("Invalid from address: {}", e)))?;

        let to: Mailbox = if email.to_name.is_empty() {
            email.to_email.parse()
        } else {
            format!("{} <{}>", email.to_name, email.to_email).parse()
        }
        .map_err(|e| NotificationError::ProviderError(format!("Invalid to address: {}", e)))?;

        let mut builder = Message::builder()
            .from(from)
            .to(to)
            .subject(&email.subject);

        // Add Reply-To if specified
        if let Some(reply_to) = &email.reply_to {
            let reply_to_mailbox: Mailbox = reply_to
                .parse()
                .map_err(|e| NotificationError::ProviderError(format!("Invalid reply-to address: {}", e)))?;
            builder = builder.reply_to(reply_to_mailbox);
        }

        // Add CC recipients
        for cc in &email.cc {
            let cc_mailbox: Mailbox = cc
                .parse()
                .map_err(|e| NotificationError::ProviderError(format!("Invalid CC address '{}': {}", cc, e)))?;
            builder = builder.cc(cc_mailbox);
        }

        // Add BCC recipients
        for bcc in &email.bcc {
            let bcc_mailbox: Mailbox = bcc
                .parse()
                .map_err(|e| NotificationError::ProviderError(format!("Invalid BCC address '{}': {}", bcc, e)))?;
            builder = builder.bcc(bcc_mailbox);
        }

        // Build multipart message with both text and HTML
        let message = builder
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(email.text_body.clone()),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(email.html_body.clone()),
                    ),
            )
            .map_err(|e| NotificationError::ProviderError(format!("Failed to build email message: {}", e)))?;

        Ok(message)
    }
}

#[async_trait]
impl EmailProvider for SmtpProvider {
    async fn send(&self, email: &EmailContent) -> NotificationResult<SentEmail> {
        debug!(
            to = %email.to_email,
            subject = %email.subject,
            host = %self.config.host,
            port = %self.config.port,
            cc_count = email.cc.len(),
            bcc_count = email.bcc.len(),
            has_reply_to = email.reply_to.is_some(),
            "Sending email via SMTP"
        );

        let message = self.build_message(email)?;

        let response = self
            .transport
            .send(message)
            .await
            .map_err(|e| {
                error!(
                    to = %email.to_email,
                    error = %e,
                    "Failed to send email via SMTP"
                );
                NotificationError::ProviderError(format!("SMTP send failed: {}", e))
            })?;

        // Extract message ID from response
        let message_id = response
            .message()
            .next()
            .map(|s| s.to_string());

        info!(
            to = %email.to_email,
            message_id = ?message_id,
            "Email sent successfully via SMTP"
        );

        Ok(SentEmail {
            message_id,
            accepted: true,
        })
    }

    fn name(&self) -> &'static str {
        "SMTP"
    }

    async fn health_check(&self) -> NotificationResult<bool> {
        self.transport
            .test_connection()
            .await
            .map_err(|e| NotificationError::ProviderError(format!("SMTP health check failed: {}", e)))?;
        Ok(true)
    }
}

// Implement Clone manually since AsyncSmtpTransport doesn't implement Clone
impl Clone for SmtpProvider {
    fn clone(&self) -> Self {
        // Rebuild transport from config
        let transport = Self::build_transport(&self.config)
            .expect("Failed to rebuild SMTP transport for clone");
        Self {
            transport,
            config: Arc::clone(&self.config),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smtp_config_mailhog() {
        let config = SmtpConfig::mailhog();
        assert_eq!(config.port, 1025);
        assert!(!config.use_tls);
    }

    #[test]
    fn test_smtp_config_new() {
        let config = SmtpConfig::new(
            "mail.example.com".to_string(),
            587,
            "test@example.com".to_string(),
            "Test".to_string(),
        );
        assert_eq!(config.host, "mail.example.com");
        assert_eq!(config.port, 587);
        assert!(!config.use_tls);
    }

    #[test]
    fn test_smtp_config_with_tls() {
        let config = SmtpConfig::new(
            "smtp.gmail.com".to_string(),
            587,
            "test@gmail.com".to_string(),
            "Test".to_string(),
        )
        .with_tls(true)
        .with_credentials("user".to_string(), "pass".to_string());

        assert!(config.use_tls);
        assert_eq!(config.username, Some("user".to_string()));
        assert_eq!(config.password, Some("pass".to_string()));
    }
}
