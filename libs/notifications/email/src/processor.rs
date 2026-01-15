//! EmailProcessor - Implements processors for both Redis and NATS backends
//!
//! This module provides the processor that handles EmailJob processing.
//! Implements both `stream_worker::StreamProcessor` (Redis) and
//! `messaging::Processor` (NATS).

use crate::job::{EmailJob, EmailType};
use crate::provider::{EmailProvider, SendResult};
use crate::templates::TemplateEngine;
use crate::Email;
use async_trait::async_trait;
use messaging::ProcessingError;
use std::sync::Arc;
use stream_worker::{StreamError, StreamProcessor};
use tracing::{debug, info};

/// Email processor that sends emails using a provider
pub struct EmailProcessor<P: EmailProvider> {
    provider: Arc<P>,
    templates: Arc<TemplateEngine>,
    from_email: String,
    from_name: String,
}

impl<P: EmailProvider> EmailProcessor<P> {
    /// Create a new EmailProcessor
    pub fn new(provider: P, templates: TemplateEngine) -> Self {
        Self {
            provider: Arc::new(provider),
            templates: Arc::new(templates),
            from_email: std::env::var("EMAIL_FROM_ADDRESS")
                .unwrap_or_else(|_| "noreply@example.com".to_string()),
            from_name: std::env::var("EMAIL_FROM_NAME")
                .unwrap_or_else(|_| "Notifications".to_string()),
        }
    }

    /// Create with explicit from address
    pub fn with_from(mut self, email: impl Into<String>, name: impl Into<String>) -> Self {
        self.from_email = email.into();
        self.from_name = name.into();
        self
    }

    /// Get the template name for an email type
    fn template_name(email_type: &EmailType) -> Option<&'static str> {
        match email_type {
            EmailType::Welcome => Some("welcome"),
            EmailType::Verification => Some("verification"),
            EmailType::PasswordReset => Some("password_reset"),
            EmailType::PasswordChanged => Some("password_changed"),
            EmailType::TaskNotification => Some("task_notification"),
            EmailType::Transactional => None,
            EmailType::Custom(name) => {
                // For custom templates, we'd need to store and return the name
                // For now, return None and let the job use body_text/body_html
                debug!(template = %name, "Custom template requested");
                None
            }
        }
    }

    /// Render an email job into a sendable Email
    fn render_job(&self, job: &EmailJob) -> Result<Email, StreamError> {
        let template_name = Self::template_name(&job.email_type);

        let (subject, body_text, body_html) = if let Some(name) = template_name {
            // Render template
            let rendered = self
                .templates
                .render(name, &job.template_vars)
                .map_err(|e| StreamError::permanent(format!("Template error: {}", e)))?;

            (rendered.subject, rendered.body_text, rendered.body_html)
        } else {
            // Use direct body from job
            (
                job.subject.clone(),
                job.body_text.clone(),
                job.body_html.clone(),
            )
        };

        // Ensure we have at least text or HTML
        if body_text.is_none() && body_html.is_none() {
            return Err(StreamError::permanent(
                "Email must have either text or HTML body",
            ));
        }

        let mut email = Email::new(&job.to_email, subject);
        email.from = Some(format!("{} <{}>", self.from_name, self.from_email));

        if let Some(text) = body_text {
            email.body_text = Some(text);
        }
        if let Some(html) = body_html {
            email.body_html = Some(html);
        }

        email.priority = job.priority.clone();
        email.retry_count = job.retry_count;

        Ok(email)
    }

    /// Send an email and handle the result (for StreamProcessor)
    async fn send_email(&self, email: &Email) -> Result<SendResult, StreamError> {
        self.provider.send(email).await.map_err(|e| {
            let msg = e.to_string();
            // Classify errors
            if msg.contains("rate limit") || msg.contains("429") {
                StreamError::rate_limited(msg)
            } else if msg.contains("invalid") || msg.contains("malformed") {
                StreamError::permanent(msg)
            } else {
                StreamError::transient(msg)
            }
        })
    }

    /// Send an email and handle the result (for messaging::Processor)
    async fn send_email_messaging(&self, email: &Email) -> Result<SendResult, ProcessingError> {
        self.provider.send(email).await.map_err(|e| {
            let msg = e.to_string();
            // Classify errors
            if msg.contains("rate limit") || msg.contains("429") {
                ProcessingError::rate_limited(msg)
            } else if msg.contains("invalid") || msg.contains("malformed") {
                ProcessingError::permanent(msg)
            } else {
                ProcessingError::transient(msg)
            }
        })
    }

    /// Render job for messaging::Processor (returns ProcessingError)
    fn render_job_messaging(&self, job: &EmailJob) -> Result<Email, ProcessingError> {
        let template_name = Self::template_name(&job.email_type);

        let (subject, body_text, body_html) = if let Some(name) = template_name {
            // Render template
            let rendered = self
                .templates
                .render(name, &job.template_vars)
                .map_err(|e| ProcessingError::permanent(format!("Template error: {}", e)))?;

            (rendered.subject, rendered.body_text, rendered.body_html)
        } else {
            // Use direct body from job
            (
                job.subject.clone(),
                job.body_text.clone(),
                job.body_html.clone(),
            )
        };

        // Ensure we have at least text or HTML
        if body_text.is_none() && body_html.is_none() {
            return Err(ProcessingError::permanent(
                "Email must have either text or HTML body",
            ));
        }

        let mut email = Email::new(&job.to_email, subject);
        email.from = Some(format!("{} <{}>", self.from_name, self.from_email));

        if let Some(text) = body_text {
            email.body_text = Some(text);
        }
        if let Some(html) = body_html {
            email.body_html = Some(html);
        }

        email.priority = job.priority.clone();
        email.retry_count = job.retry_count;

        Ok(email)
    }
}

// =============================================================================
// Redis Streams Backend (StreamProcessor)
// =============================================================================

#[async_trait]
impl<P: EmailProvider + 'static> StreamProcessor<EmailJob> for EmailProcessor<P> {
    async fn process(&self, job: &EmailJob) -> Result<(), StreamError> {
        debug!(
            job_id = %job.id,
            email_type = ?job.email_type,
            to = %job.to_email,
            "Processing email job (Redis)"
        );

        // Render the email
        let email = self.render_job(job)?;

        // Send it
        let result = self.send_email(&email).await?;

        info!(
            job_id = %job.id,
            message_id = %result.message_id,
            to = %job.to_email,
            "Email sent successfully"
        );

        Ok(())
    }

    fn name(&self) -> &'static str {
        "email_processor"
    }

    async fn health_check(&self) -> Result<bool, StreamError> {
        self.provider
            .health_check()
            .await
            .map(|_| true)
            .map_err(|e| StreamError::transient(e.to_string()))
    }
}

// =============================================================================
// NATS JetStream Backend (messaging::Processor)
// =============================================================================

#[async_trait]
impl<P: EmailProvider + 'static> messaging::Processor<EmailJob> for EmailProcessor<P> {
    async fn process(&self, job: &EmailJob) -> Result<(), ProcessingError> {
        debug!(
            job_id = %job.id,
            email_type = ?job.email_type,
            to = %job.to_email,
            "Processing email job (NATS)"
        );

        // Render the email
        let email = self.render_job_messaging(job)?;

        // Send it
        let result = self.send_email_messaging(&email).await?;

        info!(
            job_id = %job.id,
            message_id = %result.message_id,
            to = %job.to_email,
            "Email sent successfully"
        );

        Ok(())
    }

    fn name(&self) -> &'static str {
        "email_processor"
    }

    async fn health_check(&self) -> Result<bool, ProcessingError> {
        self.provider
            .health_check()
            .await
            .map(|_| true)
            .map_err(|e| ProcessingError::transient(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::MockSmtpProvider;

    #[tokio::test]
    async fn test_processor_creation() {
        let provider = MockSmtpProvider::new();
        let templates = TemplateEngine::new().unwrap();
        let processor = EmailProcessor::new(provider, templates);

        assert_eq!(processor.name(), "email_processor");
    }
}
