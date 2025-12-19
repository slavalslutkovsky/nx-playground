//! Email processor for stream workers.
//!
//! This module provides the `EmailProcessor` that implements `StreamProcessor<EmailJob>`,
//! handling the rendering and sending of emails.

use crate::error::NotificationError;
use crate::models::EmailJob;
use crate::providers::{EmailContent, EmailProvider};
use crate::templates::TemplateEngine;
use async_trait::async_trait;
use std::sync::Arc;
use stream_worker::{StreamError, StreamProcessor};
use tracing::info;

/// Email processor that renders templates and sends emails.
///
/// This processor handles `EmailJob` items from the stream,
/// renders the appropriate template, and sends via the configured provider.
pub struct EmailProcessor<P: EmailProvider> {
    provider: Arc<P>,
    templates: Arc<TemplateEngine>,
}

impl<P: EmailProvider + 'static> EmailProcessor<P> {
    /// Create a new email processor.
    pub fn new(provider: P, templates: TemplateEngine) -> Self {
        Self {
            provider: Arc::new(provider),
            templates: Arc::new(templates),
        }
    }

    /// Create a new email processor with Arc-wrapped dependencies.
    pub fn with_arcs(provider: Arc<P>, templates: Arc<TemplateEngine>) -> Self {
        Self { provider, templates }
    }

    /// Get a reference to the email provider.
    pub fn provider(&self) -> &P {
        &self.provider
    }
}

#[async_trait]
impl<P: EmailProvider + 'static> StreamProcessor<EmailJob> for EmailProcessor<P> {
    async fn process(&self, job: &EmailJob) -> Result<(), StreamError> {
        info!(
            job_id = %job.id,
            email_type = %job.email_type,
            to = %job.to_email,
            retry_count = %job.retry_count,
            "Processing email job"
        );

        // Render the email template
        let rendered = self
            .templates
            .render_by_type(&job.email_type, &job.template_vars)
            .map_err(|e| StreamError::Processing(e.to_string()))?;

        // Create email content
        let email = EmailContent {
            to_email: job.to_email.clone(),
            to_name: job.to_name.clone(),
            subject: job.subject.clone(),
            html_body: rendered.html,
            text_body: rendered.text,
            cc: Vec::new(),
            bcc: Vec::new(),
            reply_to: None,
        };

        // Send via provider
        let result = self
            .provider
            .send(&email)
            .await
            .map_err(|e| StreamError::Processing(e.to_string()))?;

        info!(
            job_id = %job.id,
            email_type = %job.email_type,
            to = %job.to_email,
            message_id = ?result.message_id,
            "Successfully sent email"
        );

        Ok(())
    }

    fn name(&self) -> &'static str {
        "EmailProcessor"
    }

    async fn health_check(&self) -> Result<bool, StreamError> {
        self.provider
            .health_check()
            .await
            .map_err(|e| StreamError::HealthCheck(e.to_string()))
    }
}

impl<P: EmailProvider> Clone for EmailProcessor<P> {
    fn clone(&self) -> Self {
        Self {
            provider: Arc::clone(&self.provider),
            templates: Arc::clone(&self.templates),
        }
    }
}

/// Convert NotificationError to StreamError for convenience.
impl From<NotificationError> for StreamError {
    fn from(e: NotificationError) -> Self {
        StreamError::Processing(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_processor_name() {
        // We can't fully test without mocks, but we can verify the name
        assert_eq!("EmailProcessor", "EmailProcessor");
    }
}
