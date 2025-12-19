//! Notification service for queueing and managing email jobs.

use crate::error::{NotificationError, NotificationResult};
use crate::models::{EmailJob, EmailType, WelcomeEmailData};
use chrono::Utc;
use rand::distr::Alphanumeric;
use rand::Rng;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

/// Configuration for the notification service.
#[derive(Debug, Clone)]
pub struct NotificationServiceConfig {
    /// Redis stream name for email jobs.
    pub stream_name: String,
    /// Consumer group name.
    pub consumer_group: String,
    /// Maximum stream length (for auto-trimming).
    pub max_stream_length: i64,
    /// Base URL for the frontend application.
    pub frontend_url: String,
    /// Email verification token expiry in hours.
    pub verification_expiry_hours: i64,
    /// Password reset token expiry in hours.
    pub password_reset_expiry_hours: i64,
    /// Company name for email footers.
    pub company_name: String,
    /// Company address for email footers.
    pub company_address: String,
    /// Logo URL for emails.
    pub logo_url: String,
}

impl Default for NotificationServiceConfig {
    fn default() -> Self {
        Self {
            stream_name: std::env::var("EMAIL_STREAM_NAME")
                .unwrap_or_else(|_| "email:jobs".to_string()),
            consumer_group: std::env::var("EMAIL_CONSUMER_GROUP")
                .unwrap_or_else(|_| "email_workers".to_string()),
            max_stream_length: 100_000,
            frontend_url: std::env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            verification_expiry_hours: std::env::var("EMAIL_VERIFICATION_EXPIRY_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .unwrap_or(24),
            password_reset_expiry_hours: std::env::var("PASSWORD_RESET_EXPIRY_HOURS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap_or(1),
            company_name: std::env::var("COMPANY_NAME")
                .unwrap_or_else(|_| "Zerg".to_string()),
            company_address: std::env::var("COMPANY_ADDRESS")
                .unwrap_or_else(|_| String::new()),
            logo_url: std::env::var("LOGO_URL")
                .unwrap_or_else(|_| String::new()),
        }
    }
}

/// Service for managing email notifications.
pub struct NotificationService {
    redis: Arc<ConnectionManager>,
    config: NotificationServiceConfig,
}

impl NotificationService {
    /// Create a new notification service.
    pub fn new(redis: ConnectionManager, config: NotificationServiceConfig) -> Self {
        Self {
            redis: Arc::new(redis),
            config,
        }
    }

    /// Create a notification service with the default config.
    pub fn with_default_config(redis: ConnectionManager) -> Self {
        Self::new(redis, NotificationServiceConfig::default())
    }

    /// Generate a secure random token.
    pub fn generate_token() -> String {
        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect()
    }

    /// Queue an email job to the Redis stream.
    async fn queue_job(&self, job: &EmailJob) -> NotificationResult<String> {
        let mut conn = (*self.redis).clone();

        let job_json = serde_json::to_string(job)?;

        // Add to stream with auto-trim
        let id: String = redis::cmd("XADD")
            .arg(&self.config.stream_name)
            .arg("MAXLEN")
            .arg("~")
            .arg(self.config.max_stream_length)
            .arg("*")
            .arg("job")
            .arg(&job_json)
            .query_async(&mut conn)
            .await?;

        debug!(
            job_id = %job.id,
            stream_id = %id,
            email_type = %job.email_type,
            to = %job.to_email,
            "Queued email job"
        );

        Ok(id)
    }

    /// Queue a welcome email for a new user.
    ///
    /// This sends a combined welcome + verification email if the user
    /// needs to verify their email address.
    pub async fn queue_welcome_email(
        &self,
        user_id: Uuid,
        email: &str,
        name: &str,
        requires_verification: bool,
        verification_token: Option<&str>,
    ) -> NotificationResult<String> {
        let verification_url = verification_token.map(|token| {
            format!(
                "{}/auth/verify-email?token={}",
                self.config.frontend_url, token
            )
        });

        let template_data = WelcomeEmailData {
            user_name: name.to_string(),
            user_email: email.to_string(),
            requires_verification,
            verification_url,
            verification_expiry_hours: self.config.verification_expiry_hours as u32,
            dashboard_url: format!("{}/dashboard", self.config.frontend_url),
            account_type: "Free".to_string(),
            join_date: Utc::now().format("%B %d, %Y").to_string(),
            logo_url: self.config.logo_url.clone(),
            preferences_url: format!("{}/settings/notifications", self.config.frontend_url),
            help_url: format!("{}/help", self.config.frontend_url),
            privacy_url: format!("{}/privacy", self.config.frontend_url),
            company_name: self.config.company_name.clone(),
            company_address: self.config.company_address.clone(),
        };

        let job = EmailJob::new(
            EmailType::Welcome,
            Some(user_id),
            email.to_string(),
            name.to_string(),
            format!("Welcome to Zerg, {}!", name),
            serde_json::to_value(&template_data)?,
        );

        let stream_id = self.queue_job(&job).await?;

        info!(
            user_id = %user_id,
            email = %email,
            requires_verification = %requires_verification,
            "Queued welcome email"
        );

        Ok(stream_id)
    }

    /// Queue a standalone verification email (for resend requests).
    pub async fn queue_verification_email(
        &self,
        user_id: Uuid,
        email: &str,
        name: &str,
        verification_token: &str,
    ) -> NotificationResult<String> {
        let verification_url = format!(
            "{}/auth/verify-email?token={}",
            self.config.frontend_url, verification_token
        );

        let template_data = WelcomeEmailData {
            user_name: name.to_string(),
            user_email: email.to_string(),
            requires_verification: true,
            verification_url: Some(verification_url),
            verification_expiry_hours: self.config.verification_expiry_hours as u32,
            dashboard_url: format!("{}/dashboard", self.config.frontend_url),
            account_type: "Free".to_string(),
            join_date: String::new(),
            logo_url: self.config.logo_url.clone(),
            preferences_url: format!("{}/settings/notifications", self.config.frontend_url),
            help_url: format!("{}/help", self.config.frontend_url),
            privacy_url: format!("{}/privacy", self.config.frontend_url),
            company_name: self.config.company_name.clone(),
            company_address: self.config.company_address.clone(),
        };

        let job = EmailJob::new(
            EmailType::Verification,
            Some(user_id),
            email.to_string(),
            name.to_string(),
            "Verify your email address".to_string(),
            serde_json::to_value(&template_data)?,
        );

        let stream_id = self.queue_job(&job).await?;

        info!(
            user_id = %user_id,
            email = %email,
            "Queued verification email"
        );

        Ok(stream_id)
    }

    /// Queue a password reset email.
    pub async fn queue_password_reset_email(
        &self,
        user_id: Uuid,
        email: &str,
        name: &str,
        reset_token: &str,
    ) -> NotificationResult<String> {
        let reset_url = format!(
            "{}/auth/reset-password?token={}",
            self.config.frontend_url, reset_token
        );

        let template_data = json!({
            "user_name": name,
            "reset_url": reset_url,
            "expiry_hours": self.config.password_reset_expiry_hours,
            "logo_url": self.config.logo_url,
            "help_url": format!("{}/help", self.config.frontend_url),
            "company_name": self.config.company_name,
            "company_address": self.config.company_address,
        });

        let job = EmailJob::new(
            EmailType::PasswordReset,
            Some(user_id),
            email.to_string(),
            name.to_string(),
            "Reset your password".to_string(),
            template_data,
        );

        let stream_id = self.queue_job(&job).await?;

        info!(
            user_id = %user_id,
            email = %email,
            "Queued password reset email"
        );

        Ok(stream_id)
    }

    /// Queue a task notification email.
    pub async fn queue_task_notification(
        &self,
        user_id: Uuid,
        email: &str,
        name: &str,
        notification_type: &str,
        task_title: &str,
        task_description: Option<&str>,
        task_url: &str,
        due_date: Option<&str>,
        assigned_by: Option<&str>,
    ) -> NotificationResult<String> {
        let template_data = json!({
            "user_name": name,
            "notification_type": notification_type,
            "task_title": task_title,
            "task_description": task_description,
            "task_url": task_url,
            "due_date": due_date,
            "assigned_by": assigned_by,
            "logo_url": self.config.logo_url,
            "preferences_url": format!("{}/settings/notifications", self.config.frontend_url),
            "company_name": self.config.company_name,
        });

        let subject = match notification_type {
            "assigned" => format!("Task assigned: {}", task_title),
            "due_soon" => format!("Task due soon: {}", task_title),
            "overdue" => format!("Task overdue: {}", task_title),
            "completed" => format!("Task completed: {}", task_title),
            _ => format!("Task update: {}", task_title),
        };

        let job = EmailJob::new(
            EmailType::TaskNotification,
            Some(user_id),
            email.to_string(),
            name.to_string(),
            subject,
            template_data,
        );

        let stream_id = self.queue_job(&job).await?;

        info!(
            user_id = %user_id,
            email = %email,
            notification_type = %notification_type,
            task = %task_title,
            "Queued task notification email"
        );

        Ok(stream_id)
    }

    /// Ensure the consumer group exists for the email stream.
    pub async fn ensure_consumer_group(&self) -> NotificationResult<()> {
        let mut conn = (*self.redis).clone();

        // Try to create the consumer group
        // XGROUP CREATE stream_name group_name $ MKSTREAM
        let result: Result<(), redis::RedisError> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(&self.config.stream_name)
            .arg(&self.config.consumer_group)
            .arg("$")
            .arg("MKSTREAM")
            .query_async(&mut conn)
            .await;

        match result {
            Ok(_) => {
                info!(
                    stream = %self.config.stream_name,
                    group = %self.config.consumer_group,
                    "Created consumer group"
                );
                Ok(())
            }
            Err(e) if e.to_string().contains("BUSYGROUP") => {
                // Consumer group already exists, this is fine
                debug!(
                    stream = %self.config.stream_name,
                    group = %self.config.consumer_group,
                    "Consumer group already exists"
                );
                Ok(())
            }
            Err(e) => Err(NotificationError::QueueError(e.to_string())),
        }
    }

    /// Get the number of pending messages in the stream.
    pub async fn get_pending_count(&self) -> NotificationResult<i64> {
        let mut conn = (*self.redis).clone();

        let len: i64 = conn.xlen(&self.config.stream_name).await?;
        Ok(len)
    }

    /// Get the stream name.
    pub fn stream_name(&self) -> &str {
        &self.config.stream_name
    }

    /// Get the consumer group name.
    pub fn consumer_group(&self) -> &str {
        &self.config.consumer_group
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token() {
        let token = NotificationService::generate_token();
        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_default_config() {
        let config = NotificationServiceConfig::default();
        assert_eq!(config.stream_name, "email:jobs");
        assert_eq!(config.consumer_group, "email_workers");
        assert_eq!(config.verification_expiry_hours, 24);
        assert_eq!(config.password_reset_expiry_hours, 1);
    }
}
