//! Mock email provider for testing

use super::{EmailProvider, SendResult};
use crate::models::Email;
use async_trait::async_trait;
use eyre::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Mock email provider that captures sent emails
pub struct MockSmtpProvider {
    sent_emails: Arc<Mutex<Vec<Email>>>,
    should_fail: bool,
    failure_message: Option<String>,
}

impl MockSmtpProvider {
    /// Create a new mock provider
    pub fn new() -> Self {
        Self {
            sent_emails: Arc::new(Mutex::new(Vec::new())),
            should_fail: false,
            failure_message: None,
        }
    }

    /// Create a mock provider that always fails
    pub fn failing(message: impl Into<String>) -> Self {
        Self {
            sent_emails: Arc::new(Mutex::new(Vec::new())),
            should_fail: true,
            failure_message: Some(message.into()),
        }
    }

    /// Get all sent emails
    pub async fn sent_emails(&self) -> Vec<Email> {
        self.sent_emails.lock().await.clone()
    }

    /// Get the count of sent emails
    pub async fn sent_count(&self) -> usize {
        self.sent_emails.lock().await.len()
    }

    /// Clear all sent emails
    pub async fn clear(&self) {
        self.sent_emails.lock().await.clear();
    }

    /// Check if an email was sent to a specific address
    pub async fn was_sent_to(&self, email: &str) -> bool {
        self.sent_emails
            .lock()
            .await
            .iter()
            .any(|e| e.to == email)
    }
}

impl Default for MockSmtpProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EmailProvider for MockSmtpProvider {
    async fn send(&self, email: &Email) -> Result<SendResult> {
        if self.should_fail {
            let message = self
                .failure_message
                .clone()
                .unwrap_or_else(|| "Mock failure".to_string());
            return Err(eyre::eyre!(message));
        }

        self.sent_emails.lock().await.push(email.clone());

        Ok(SendResult {
            message_id: format!("mock-{}", email.id),
        })
    }

    async fn health_check(&self) -> Result<()> {
        if self.should_fail {
            return Err(eyre::eyre!("Mock health check failed"));
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "mock"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_sends_email() {
        let provider = MockSmtpProvider::new();

        let email = Email::new("test@example.com", "Test Subject")
            .with_text("Test body");

        let result = provider.send(&email).await;
        assert!(result.is_ok());

        let sent = provider.sent_emails().await;
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].to, "test@example.com");
    }

    #[tokio::test]
    async fn test_mock_provider_fails() {
        let provider = MockSmtpProvider::failing("Simulated failure");

        let email = Email::new("test@example.com", "Test Subject")
            .with_text("Test body");

        let result = provider.send(&email).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Simulated failure"));
    }

    #[tokio::test]
    async fn test_mock_provider_was_sent_to() {
        let provider = MockSmtpProvider::new();

        let email = Email::new("user@example.com", "Test").with_text("Body");
        provider.send(&email).await.unwrap();

        assert!(provider.was_sent_to("user@example.com").await);
        assert!(!provider.was_sent_to("other@example.com").await);
    }
}
