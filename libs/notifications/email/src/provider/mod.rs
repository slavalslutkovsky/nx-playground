//! Email provider implementations
//!
//! Available providers:
//!
//! | Provider | Use Case | Auth Method |
//! |----------|----------|-------------|
//! | [`SendGridProvider`] | General purpose | API Key |
//! | [`SesProvider`] | AWS-native apps | AWS credentials / IRSA |
//! | [`SesVaultProvider`] | AWS SES + Vault | Dynamic Vault credentials |
//! | [`GmailProvider`] | Google Workspace | Service Account + Delegation |
//! | [`GmailOAuth2Provider`] | Any Gmail account | OAuth2 Refresh Token |
//! | [`SmtpProvider`] | Generic SMTP | Username/Password |
//!
//! ## Gmail via SMTP
//!
//! For simpler Gmail integration, use [`SmtpProvider`] with these helpers:
//! - [`SmtpProvider::gmail_app_password()`] - Personal Gmail with app password
//! - [`SmtpProvider::gmail_relay()`] - Workspace SMTP relay (IP allowlist)
//! - [`SmtpProvider::gmail_relay_with_auth()`] - Workspace SMTP relay with auth
//!
//! ## Vault Integration (requires `vault` feature)
//!
//! For dynamic AWS credentials via HashiCorp Vault:
//! - [`SesVaultProvider`] - AWS SES with Vault-managed credentials

pub mod gmail;
pub mod gmail_oauth2;
pub mod mock;
pub mod sendgrid;
pub mod ses;
#[cfg(feature = "vault")]
pub mod ses_vault;
pub mod smtp;

pub use gmail::{GmailProvider, ServiceAccountKey};
pub use gmail_oauth2::GmailOAuth2Provider;
pub use mock::MockSmtpProvider;
pub use sendgrid::SendGridProvider;
pub use ses::SesProvider;
#[cfg(feature = "vault")]
pub use ses_vault::{SesVaultConfig, SesVaultProvider};
pub use smtp::{SmtpConfig, SmtpProvider};

use crate::models::Email;
use async_trait::async_trait;
use eyre::Result;

/// Result of sending an email
#[derive(Debug)]
pub struct SendResult {
    /// Provider-specific message ID
    pub message_id: String,
}

/// Trait for email providers
#[async_trait]
pub trait EmailProvider: Send + Sync {
    /// Send an email
    async fn send(&self, email: &Email) -> Result<SendResult>;

    /// Check if the provider is healthy
    async fn health_check(&self) -> Result<()>;

    /// Get provider name
    fn name(&self) -> &'static str;
}
