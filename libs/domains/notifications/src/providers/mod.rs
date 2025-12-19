//! Email provider implementations.
//!
//! This module contains the `EmailProvider` trait and implementations
//! for different email sending services.

mod sendgrid;
mod smtp;

pub use sendgrid::SendGridProvider;
pub use smtp::SmtpProvider;

use crate::error::NotificationResult;
use async_trait::async_trait;

/// Represents a sent email with provider-specific message ID.
#[derive(Debug, Clone)]
pub struct SentEmail {
    /// Provider-specific message ID for tracking.
    pub message_id: Option<String>,
    /// Whether the email was accepted for delivery.
    pub accepted: bool,
}

/// Email content ready for sending.
#[derive(Debug, Clone, Default)]
pub struct EmailContent {
    /// Recipient email address.
    pub to_email: String,
    /// Recipient name.
    pub to_name: String,
    /// Email subject.
    pub subject: String,
    /// HTML body content.
    pub html_body: String,
    /// Plain text body content.
    pub text_body: String,
    /// CC recipients (email addresses).
    pub cc: Vec<String>,
    /// BCC recipients (email addresses).
    pub bcc: Vec<String>,
    /// Reply-To email address.
    pub reply_to: Option<String>,
}

/// Trait for email sending providers.
///
/// Implementations include SendGrid, SMTP, AWS SES, etc.
#[async_trait]
pub trait EmailProvider: Send + Sync {
    /// Send an email.
    async fn send(&self, email: &EmailContent) -> NotificationResult<SentEmail>;

    /// Get the provider name for logging.
    fn name(&self) -> &'static str;

    /// Check if the provider is healthy/configured.
    async fn health_check(&self) -> NotificationResult<bool>;
}
