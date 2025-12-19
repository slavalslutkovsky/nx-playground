//! Notifications Domain
//!
//! This module provides email notification services for the application.
//!
//! # Features
//!
//! - Welcome emails for new users
//! - Email verification flow
//! - Password reset emails
//! - Task notification emails
//! - Weekly digest emails
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────┐
//! │   API Handler   │  ← Queues email jobs
//! └────────┬────────┘
//!          │
//! ┌────────▼────────┐
//! │ NotificationSvc │  ← Queues jobs to Redis Stream
//! └────────┬────────┘
//!          │
//! ┌────────▼────────┐
//! │   Redis Stream  │  ← email:jobs queue
//! └────────┬────────┘
//!          │
//! ┌────────▼────────┐
//! │  Email Worker   │  ← Consumes and processes jobs
//! └────────┬────────┘
//!          │
//! ┌────────▼────────┐
//! │ Email Provider  │  ← SendGrid, SMTP, etc.
//! └─────────────────┘
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use domain_notifications::{
//!     NotificationService,
//!     EmailJob,
//!     providers::SendGridProvider,
//! };
//!
//! // Create notification service
//! let service = NotificationService::new(redis_pool);
//!
//! // Queue a welcome email
//! service.queue_welcome_email(user_id, &email, &name).await?;
//! ```

pub mod error;
pub mod models;
pub mod processor;
pub mod providers;
pub mod service;
pub mod streams;
pub mod templates;
pub mod worker;

// Re-export commonly used types
pub use error::{NotificationError, NotificationResult};
pub use models::{
    EmailJob, EmailLog, EmailLogStatus, EmailPreferences, EmailSuppression, EmailType,
    VerificationToken, VerificationTokenType,
};
pub use processor::EmailProcessor;
pub use providers::{EmailProvider, SendGridProvider, SmtpProvider};
pub use service::NotificationService;
pub use streams::EmailStream;
pub use templates::TemplateEngine;
