//! AWS SES (Simple Email Service) provider
//!
//! Sends emails via AWS SES v2 API.
//!
//! ## Configuration
//!
//! The provider uses standard AWS SDK credential resolution:
//! - Environment variables: `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_REGION`
//! - IAM roles (EKS IRSA, EC2 instance profile)
//! - Shared credentials file
//!
//! Required environment variables:
//! - `AWS_REGION` or `AWS_SES_REGION` - AWS region for SES
//! - `SES_FROM_EMAIL` or `EMAIL_FROM_ADDRESS` - Default sender email
//! - `SES_FROM_NAME` or `EMAIL_FROM_NAME` - Default sender name (optional)

use crate::models::Email;
use crate::provider::{EmailProvider, SendResult};
use async_trait::async_trait;
use aws_sdk_sesv2::types::{Body, Content, Destination, EmailContent, Message};
use aws_sdk_sesv2::Client;
use eyre::{eyre, Result};
use tracing::{debug, error};

/// AWS SES email provider
pub struct SesProvider {
    client: Client,
    from_email: String,
    from_name: String,
}

impl SesProvider {
    /// Create a new SesProvider with an existing AWS SES client
    pub fn new(
        client: Client,
        from_email: impl Into<String>,
        from_name: impl Into<String>,
    ) -> Self {
        Self {
            client,
            from_email: from_email.into(),
            from_name: from_name.into(),
        }
    }

    /// Create from environment variables and default AWS SDK config
    ///
    /// Uses AWS SDK's default credential chain:
    /// - Environment variables (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`)
    /// - Web identity token (EKS IRSA)
    /// - IAM instance profile (EC2)
    /// - Shared credentials file
    pub async fn from_env() -> Result<Self> {
        // Check for region override
        let region = std::env::var("AWS_SES_REGION")
            .or_else(|_| std::env::var("AWS_REGION"))
            .ok();

        let mut config_loader = aws_config::from_env();

        if let Some(region_str) = region {
            config_loader = config_loader.region(aws_config::Region::new(region_str));
        }

        let config = config_loader.load().await;
        let client = Client::new(&config);

        let from_email = std::env::var("SES_FROM_EMAIL")
            .or_else(|_| std::env::var("EMAIL_FROM_ADDRESS"))
            .map_err(|_| eyre!("SES_FROM_EMAIL or EMAIL_FROM_ADDRESS not set"))?;

        let from_name = std::env::var("SES_FROM_NAME")
            .or_else(|_| std::env::var("EMAIL_FROM_NAME"))
            .unwrap_or_else(|_| "Notifications".to_string());

        Ok(Self::new(client, from_email, from_name))
    }

    /// Create with explicit credentials (useful for testing)
    pub async fn with_credentials(
        access_key_id: impl Into<String>,
        secret_access_key: impl Into<String>,
        region: impl Into<String>,
        from_email: impl Into<String>,
        from_name: impl Into<String>,
    ) -> Result<Self> {
        let credentials = aws_sdk_sesv2::config::Credentials::new(
            access_key_id,
            secret_access_key,
            None, // session token
            None, // expiry
            "manual",
        );

        let config = aws_sdk_sesv2::Config::builder()
            .region(aws_sdk_sesv2::config::Region::new(region.into()))
            .credentials_provider(credentials)
            .build();

        let client = Client::from_conf(config);

        Ok(Self::new(client, from_email, from_name))
    }

    /// Format email address with name
    fn format_address(&self, email: &str, name: Option<&str>) -> String {
        match name {
            Some(n) if !n.is_empty() => format!("{} <{}>", n, email),
            _ => email.to_string(),
        }
    }
}

#[async_trait]
impl EmailProvider for SesProvider {
    async fn send(&self, email: &Email) -> Result<SendResult> {
        // Build destination
        let mut destination = Destination::builder().to_addresses(&email.to);

        for cc in &email.cc {
            destination = destination.cc_addresses(cc);
        }

        for bcc in &email.bcc {
            destination = destination.bcc_addresses(bcc);
        }

        let destination = destination.build();

        // Build body
        let mut body = Body::builder();

        if let Some(text) = &email.body_text {
            body = body.text(Content::builder().data(text).charset("UTF-8").build()?);
        }

        if let Some(html) = &email.body_html {
            body = body.html(Content::builder().data(html).charset("UTF-8").build()?);
        }

        let body = body.build();

        // Build message
        let message = Message::builder()
            .subject(
                Content::builder()
                    .data(&email.subject)
                    .charset("UTF-8")
                    .build()?,
            )
            .body(body)
            .build();

        // Build email content
        let email_content = EmailContent::builder().simple(message).build();

        // Get from address
        let from_email = email.from.as_ref().unwrap_or(&self.from_email);
        let from_address = self.format_address(from_email, Some(&self.from_name));

        debug!(
            to = %email.to,
            subject = %email.subject,
            from = %from_address,
            "Sending email via AWS SES"
        );

        // Send the email
        let mut request = self
            .client
            .send_email()
            .from_email_address(&from_address)
            .destination(destination)
            .content(email_content);

        // Add reply-to if specified
        if let Some(reply_to) = &email.reply_to {
            request = request.reply_to_addresses(reply_to);
        }

        let response = request.send().await.map_err(|e| {
            error!(error = %e, "AWS SES send failed");

            // Map SES errors to appropriate categories
            let err_str = e.to_string();
            if err_str.contains("Throttling") || err_str.contains("rate") {
                eyre!("rate limit exceeded: {}", e)
            } else if err_str.contains("AccessDenied") || err_str.contains("credentials") {
                eyre!("authentication failed: {}", e)
            } else if err_str.contains("ValidationError") || err_str.contains("InvalidParameter") {
                eyre!("invalid request: {}", e)
            } else {
                eyre!("SES error: {}", e)
            }
        })?;

        let message_id = response.message_id().unwrap_or(&email.id).to_string();

        debug!(
            message_id = %message_id,
            "Email sent successfully via AWS SES"
        );

        Ok(SendResult { message_id })
    }

    async fn health_check(&self) -> Result<()> {
        // Verify we can access the SES service
        // GetAccount is a lightweight call that confirms credentials and access
        self.client
            .get_account()
            .send()
            .await
            .map_err(|e| eyre!("AWS SES health check failed: {}", e))?;

        Ok(())
    }

    fn name(&self) -> &'static str {
        "aws-ses"
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_format_address_logic() {
        // Test the format address logic without needing an actual SesProvider
        fn format_address(email: &str, name: Option<&str>) -> String {
            match name {
                Some(n) if !n.is_empty() => format!("{} <{}>", n, email),
                _ => email.to_string(),
            }
        }

        assert_eq!(
            format_address("test@example.com", Some("Test User")),
            "Test User <test@example.com>"
        );
        assert_eq!(
            format_address("test@example.com", None),
            "test@example.com"
        );
        assert_eq!(
            format_address("test@example.com", Some("")),
            "test@example.com"
        );
    }
}
