//! Email notification library with Redis Streams support
//!
//! This library provides a complete email notification system:
//!
//! ## Features
//!
//! - **Stream Processing**: `EmailJob`, `EmailStream`, `EmailProcessor` for use with `stream-worker`
//! - **Email Models**: `Email`, `EmailEvent`, `EmailPriority` for email data
//! - **Providers**: SMTP, SendGrid, and Mock providers
//! - **Templates**: Handlebars-based `TemplateEngine` for email templating
//!
//! ## Usage with stream-worker
//!
//! ```ignore
//! use email::{EmailJob, EmailStream, EmailProcessor};
//! use email::provider::SmtpProvider;
//! use email::templates::TemplateEngine;
//! use stream_worker::{StreamWorker, WorkerConfig};
//!
//! // Create processor
//! let provider = SmtpProvider::from_env()?;
//! let templates = TemplateEngine::new()?;
//! let processor = EmailProcessor::new(provider, templates);
//!
//! // Create worker
//! let config = WorkerConfig::from_stream_def::<EmailStream>();
//! let worker = StreamWorker::new(redis, processor, config);
//!
//! // Run
//! worker.run(shutdown_rx).await?;
//! ```

// Core modules
pub mod error;
pub mod job;
pub mod models;
pub mod processor;
pub mod provider;
pub mod service;
pub mod stream;
pub mod streams;
pub mod templates;

// Re-export main types
pub use error::{NotificationError, NotificationResult};
pub use job::{EmailJob, EmailType};
pub use models::{Email, EmailEvent, EmailPriority, EmailStatus};
pub use processor::EmailProcessor;
pub use provider::{EmailProvider, MockSmtpProvider, SendGridProvider, SendResult, SmtpProvider};
pub use service::{NotificationService, NotificationServiceConfig, WelcomeEmailData};
pub use stream::{get_stream_info, EmailConsumer, EmailProducer, StreamInfo};
pub use streams::EmailStream;
pub use templates::{InMemoryTemplateStore, TemplateEngine, TemplateStore};
