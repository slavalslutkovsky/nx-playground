//! Email notification library with Redis Streams and NATS JetStream support
//!
//! This library provides a complete email notification system that works with
//! both Redis Streams (`stream-worker`) and NATS JetStream (`nats-worker`).
//!
//! ## Features
//!
//! - **Stream Processing**: `EmailJob`, `EmailStream`/`EmailNatsStream`, `EmailProcessor`
//! - **Dual Backend**: Works with both Redis Streams and NATS JetStream
//! - **Email Models**: `Email`, `EmailEvent`, `EmailPriority` for email data
//! - **Providers**: SMTP, SendGrid, and Mock providers
//! - **Templates**: Handlebars-based `TemplateEngine` for email templating
//!
//! ## Usage with Redis Streams (stream-worker)
//!
//! ```ignore
//! use email::{EmailJob, EmailStream, EmailProcessor};
//! use stream_worker::{StreamWorker, WorkerConfig};
//!
//! let processor = EmailProcessor::new(provider, templates);
//! let config = WorkerConfig::from_stream_def::<EmailStream>();
//! let worker = StreamWorker::new(redis, processor, config);
//! worker.run(shutdown_rx).await?;
//! ```
//!
//! ## Usage with NATS JetStream (nats-worker)
//!
//! ```ignore
//! use email::{EmailJob, EmailNatsStream, EmailProcessor};
//! use nats_worker::{NatsWorker, WorkerConfig};
//!
//! let processor = EmailProcessor::new(provider, templates);
//! let config = WorkerConfig::from_stream::<EmailNatsStream>();
//! let worker = NatsWorker::new(jetstream, processor, config).await?;
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
pub use streams::{EmailNatsStream, EmailStream};
pub use templates::{InMemoryTemplateStore, TemplateEngine, TemplateStore};
