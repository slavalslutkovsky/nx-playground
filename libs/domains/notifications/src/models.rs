//! Data models for the notifications domain.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stream_worker::StreamJob;
use uuid::Uuid;

// ============================================================================
// Email Job Types (for Redis Stream queue)
// ============================================================================

/// Types of emails that can be sent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmailType {
    /// Welcome email sent after registration.
    Welcome,
    /// Email verification email.
    Verification,
    /// Password reset email.
    PasswordReset,
    /// Task assignment/update notification.
    TaskNotification,
    /// Weekly activity digest.
    WeeklyDigest,
}

impl std::fmt::Display for EmailType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailType::Welcome => write!(f, "welcome"),
            EmailType::Verification => write!(f, "verification"),
            EmailType::PasswordReset => write!(f, "password_reset"),
            EmailType::TaskNotification => write!(f, "task_notification"),
            EmailType::WeeklyDigest => write!(f, "weekly_digest"),
        }
    }
}

/// An email job to be processed by the worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailJob {
    /// Unique job identifier.
    pub id: Uuid,
    /// Type of email to send.
    pub email_type: EmailType,
    /// User ID (if associated with a user).
    pub user_id: Option<Uuid>,
    /// Recipient email address.
    pub to_email: String,
    /// Recipient name (for personalization).
    pub to_name: String,
    /// Email subject line.
    pub subject: String,
    /// Template variables for rendering.
    pub template_vars: serde_json::Value,
    /// Number of retry attempts so far.
    pub retry_count: u32,
    /// Job creation timestamp.
    pub created_at: DateTime<Utc>,
}

impl EmailJob {
    /// Create a new email job.
    pub fn new(
        email_type: EmailType,
        user_id: Option<Uuid>,
        to_email: String,
        to_name: String,
        subject: String,
        template_vars: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            email_type,
            user_id,
            to_email,
            to_name,
            subject,
            template_vars,
            retry_count: 0,
            created_at: Utc::now(),
        }
    }
}

impl StreamJob for EmailJob {
    fn job_id(&self) -> String {
        self.id.to_string()
    }

    fn retry_count(&self) -> u32 {
        self.retry_count
    }

    fn with_retry(&self) -> Self {
        Self {
            id: Uuid::new_v4(), // New ID for the retry
            retry_count: self.retry_count + 1,
            ..self.clone()
        }
    }

    fn max_retries(&self) -> u32 {
        3
    }
}

// ============================================================================
// Database Models
// ============================================================================

/// Status of an email log entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmailLogStatus {
    /// Email is queued for sending.
    Queued,
    /// Email has been sent to the provider.
    Sent,
    /// Email has been delivered to the recipient.
    Delivered,
    /// Email was opened by the recipient.
    Opened,
    /// Email bounced (hard or soft).
    Bounced,
    /// Email sending failed permanently.
    Failed,
}

impl std::fmt::Display for EmailLogStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailLogStatus::Queued => write!(f, "queued"),
            EmailLogStatus::Sent => write!(f, "sent"),
            EmailLogStatus::Delivered => write!(f, "delivered"),
            EmailLogStatus::Opened => write!(f, "opened"),
            EmailLogStatus::Bounced => write!(f, "bounced"),
            EmailLogStatus::Failed => write!(f, "failed"),
        }
    }
}

/// Email log entry for tracking sent emails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub email_type: EmailType,
    pub to_email: String,
    pub subject: String,
    pub status: EmailLogStatus,
    pub provider_message_id: Option<String>,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub opened_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl EmailLog {
    /// Create a new email log entry in queued status.
    pub fn new_queued(
        user_id: Option<Uuid>,
        email_type: EmailType,
        to_email: String,
        subject: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            email_type,
            to_email,
            subject,
            status: EmailLogStatus::Queued,
            provider_message_id: None,
            error_message: None,
            retry_count: 0,
            sent_at: None,
            delivered_at: None,
            opened_at: None,
            created_at: Utc::now(),
        }
    }
}

/// User email preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailPreferences {
    pub user_id: Uuid,
    pub welcome_enabled: bool,
    pub task_notifications_enabled: bool,
    pub weekly_digest_enabled: bool,
    pub marketing_enabled: bool,
    pub updated_at: DateTime<Utc>,
}

impl Default for EmailPreferences {
    fn default() -> Self {
        Self {
            user_id: Uuid::nil(),
            welcome_enabled: true,
            task_notifications_enabled: true,
            weekly_digest_enabled: true,
            marketing_enabled: false,
            updated_at: Utc::now(),
        }
    }
}

impl EmailPreferences {
    /// Create default preferences for a user.
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            ..Default::default()
        }
    }

    /// Check if a specific email type is enabled.
    pub fn is_enabled(&self, email_type: &EmailType) -> bool {
        match email_type {
            EmailType::Welcome => self.welcome_enabled,
            EmailType::Verification => true, // Always enabled (required for account security)
            EmailType::PasswordReset => true, // Always enabled (required for account security)
            EmailType::TaskNotification => self.task_notifications_enabled,
            EmailType::WeeklyDigest => self.weekly_digest_enabled,
        }
    }
}

/// Reason for email suppression.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SuppressionReason {
    /// Email bounced (invalid address).
    Bounce,
    /// Recipient marked as spam.
    Complaint,
    /// User unsubscribed.
    Unsubscribe,
}

impl std::fmt::Display for SuppressionReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SuppressionReason::Bounce => write!(f, "bounce"),
            SuppressionReason::Complaint => write!(f, "complaint"),
            SuppressionReason::Unsubscribe => write!(f, "unsubscribe"),
        }
    }
}

/// Email suppression entry (for bounced/complained/unsubscribed emails).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSuppression {
    pub id: Uuid,
    pub email: String,
    pub reason: SuppressionReason,
    pub created_at: DateTime<Utc>,
}

impl EmailSuppression {
    /// Create a new suppression entry.
    pub fn new(email: String, reason: SuppressionReason) -> Self {
        Self {
            id: Uuid::new_v4(),
            email,
            reason,
            created_at: Utc::now(),
        }
    }
}

// ============================================================================
// Verification Token Types
// ============================================================================

/// Type of verification token.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationTokenType {
    /// Email verification token.
    EmailVerification,
    /// Password reset token.
    PasswordReset,
}

impl std::fmt::Display for VerificationTokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerificationTokenType::EmailVerification => write!(f, "email_verification"),
            VerificationTokenType::PasswordReset => write!(f, "password_reset"),
        }
    }
}

/// Verification token for email verification and password reset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub token_type: VerificationTokenType,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl VerificationToken {
    /// Create a new verification token.
    pub fn new(
        user_id: Uuid,
        token: String,
        token_type: VerificationTokenType,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            token,
            token_type,
            expires_at,
            used_at: None,
            created_at: Utc::now(),
        }
    }

    /// Check if the token has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if the token has been used.
    pub fn is_used(&self) -> bool {
        self.used_at.is_some()
    }

    /// Check if the token is valid (not expired and not used).
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_used()
    }
}

// ============================================================================
// Template Data Structures
// ============================================================================

/// Data for rendering the welcome email template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WelcomeEmailData {
    pub user_name: String,
    pub user_email: String,
    pub requires_verification: bool,
    pub verification_url: Option<String>,
    pub verification_expiry_hours: u32,
    pub dashboard_url: String,
    pub account_type: String,
    pub join_date: String,
    pub logo_url: String,
    pub preferences_url: String,
    pub help_url: String,
    pub privacy_url: String,
    pub company_name: String,
    pub company_address: String,
}

/// Data for rendering the password reset email template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetEmailData {
    pub user_name: String,
    pub reset_url: String,
    pub expiry_hours: u32,
    pub logo_url: String,
    pub help_url: String,
    pub company_name: String,
    pub company_address: String,
}

/// Data for rendering task notification emails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNotificationData {
    pub user_name: String,
    pub notification_type: String, // "assigned", "due_soon", "overdue", "completed"
    pub task_title: String,
    pub task_description: Option<String>,
    pub task_url: String,
    pub due_date: Option<String>,
    pub assigned_by: Option<String>,
    pub logo_url: String,
    pub preferences_url: String,
    pub company_name: String,
}

/// Data for rendering weekly digest emails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyDigestData {
    pub user_name: String,
    pub tasks_completed: u32,
    pub tasks_created: u32,
    pub tasks_overdue: u32,
    pub upcoming_tasks: Vec<DigestTaskItem>,
    pub productivity_score: Option<u32>,
    pub dashboard_url: String,
    pub logo_url: String,
    pub preferences_url: String,
    pub company_name: String,
    pub company_address: String,
}

/// A task item for the weekly digest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestTaskItem {
    pub title: String,
    pub due_date: String,
    pub url: String,
}
