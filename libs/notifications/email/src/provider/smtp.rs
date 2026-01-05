//! SMTP email provider using lettre

use super::{EmailProvider, SendResult};
use crate::models::Email;
use async_trait::async_trait;
use eyre::{Result, WrapErr};
use lettre::{
    message::{header::ContentType, Mailbox, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use std::sync::Arc;

/// SMTP provider configuration
#[derive(Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: String,
    pub use_tls: bool,
}

/// SMTP email provider
pub struct SmtpProvider {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    config: Arc<SmtpConfig>,
}

impl SmtpProvider {
    /// Create a new SMTP provider
    pub fn new(config: SmtpConfig) -> Result<Self> {
        let transport = if config.use_tls {
            let creds = Credentials::new(config.username.clone(), config.password.clone());
            AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
                .wrap_err("Failed to create SMTP relay")?
                .credentials(creds)
                .port(config.port)
                .build()
        } else if !config.username.is_empty() {
            let creds = Credentials::new(config.username.clone(), config.password.clone());
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
                .credentials(creds)
                .port(config.port)
                .build()
        } else {
            // No auth (for Mailpit/Mailhog)
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
                .port(config.port)
                .build()
        };

        Ok(Self {
            transport,
            config: Arc::new(config),
        })
    }

    /// Create a provider for Mailhog/Mailpit (local development)
    ///
    /// Connects to localhost:1025 without authentication.
    pub fn mailhog() -> Result<Self> {
        let host = std::env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port: u16 = std::env::var("SMTP_PORT")
            .unwrap_or_else(|_| "1025".to_string())
            .parse()
            .unwrap_or(1025);

        let config = SmtpConfig {
            host,
            port,
            username: String::new(),
            password: String::new(),
            from_email: std::env::var("EMAIL_FROM_ADDRESS")
                .unwrap_or_else(|_| "noreply@localhost".to_string()),
            from_name: std::env::var("EMAIL_FROM_NAME")
                .unwrap_or_else(|_| "Development".to_string()),
            use_tls: false,
        };

        Self::new(config)
    }

    /// Create a provider from environment variables
    pub fn from_env() -> Result<Self> {
        let config = SmtpConfig {
            host: std::env::var("SMTP_HOST").wrap_err("SMTP_HOST not set")?,
            port: std::env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .wrap_err("Invalid SMTP_PORT")?,
            username: std::env::var("SMTP_USERNAME").unwrap_or_default(),
            password: std::env::var("SMTP_PASSWORD").unwrap_or_default(),
            from_email: std::env::var("EMAIL_FROM_ADDRESS")
                .or_else(|_| std::env::var("SMTP_FROM_EMAIL"))
                .wrap_err("EMAIL_FROM_ADDRESS not set")?,
            from_name: std::env::var("EMAIL_FROM_NAME").unwrap_or_else(|_| "Notifications".to_string()),
            use_tls: std::env::var("SMTP_USE_TLS")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
        };

        Self::new(config)
    }

    fn build_message(&self, email: &Email) -> Result<Message> {
        let from: Mailbox = format!("{} <{}>", self.config.from_name, self.config.from_email)
            .parse()
            .wrap_err("Invalid from address")?;

        let to: Mailbox = email.to.parse().wrap_err("Invalid to address")?;

        let mut builder = Message::builder().from(from).to(to).subject(&email.subject);

        // Add reply-to if specified
        if let Some(reply_to) = &email.reply_to {
            let reply_to_mailbox: Mailbox =
                reply_to.parse().wrap_err("Invalid reply-to address")?;
            builder = builder.reply_to(reply_to_mailbox);
        }

        // Add CC recipients
        for cc in &email.cc {
            let cc_mailbox: Mailbox = cc.parse().wrap_err("Invalid CC address")?;
            builder = builder.cc(cc_mailbox);
        }

        // Add BCC recipients
        for bcc in &email.bcc {
            let bcc_mailbox: Mailbox = bcc.parse().wrap_err("Invalid BCC address")?;
            builder = builder.bcc(bcc_mailbox);
        }

        // Build body
        let message = match (&email.body_text, &email.body_html) {
            (Some(text), Some(html)) => {
                // Multipart message with both text and HTML
                builder
                    .multipart(
                        MultiPart::alternative()
                            .singlepart(
                                SinglePart::builder()
                                    .header(ContentType::TEXT_PLAIN)
                                    .body(text.clone()),
                            )
                            .singlepart(
                                SinglePart::builder()
                                    .header(ContentType::TEXT_HTML)
                                    .body(html.clone()),
                            ),
                    )
                    .wrap_err("Failed to build multipart message")?
            }
            (Some(text), None) => builder
                .header(ContentType::TEXT_PLAIN)
                .body(text.clone())
                .wrap_err("Failed to build text message")?,
            (None, Some(html)) => builder
                .header(ContentType::TEXT_HTML)
                .body(html.clone())
                .wrap_err("Failed to build HTML message")?,
            (None, None) => {
                return Err(eyre::eyre!("Email must have either text or HTML body"));
            }
        };

        Ok(message)
    }
}

#[async_trait]
impl EmailProvider for SmtpProvider {
    async fn send(&self, email: &Email) -> Result<SendResult> {
        let message = self.build_message(email)?;

        let response = self
            .transport
            .send(message)
            .await
            .wrap_err("Failed to send email via SMTP")?;

        // Extract message ID from response
        let message_id = response
            .message()
            .next()
            .map(|s| s.to_string())
            .unwrap_or_else(|| email.id.clone());

        tracing::info!(
            email_id = %email.id,
            to = %email.to,
            subject = %email.subject,
            "Email sent successfully"
        );

        Ok(SendResult { message_id })
    }

    async fn health_check(&self) -> Result<()> {
        self.transport
            .test_connection()
            .await
            .wrap_err("SMTP health check failed")?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "smtp"
    }
}
