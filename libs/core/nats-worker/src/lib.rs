//! NATS JetStream worker framework for background job processing.
//!
//! This library provides a production-ready worker framework using NATS JetStream
//! for durable message queues. It implements the same `Job` and `Processor` traits
//! as `stream-worker` (Redis Streams), making it easy to switch backends.
//!
//! # Architecture
//!
//! ```text
//! ┌────────────────┐     ┌─────────────────────┐     ┌────────────────┐
//! │   Producer     │────▶│   NATS JetStream    │────▶│     Worker     │
//! │ (NatsProducer) │     │   (Durable Stream)  │     │ (NatsWorker)   │
//! └────────────────┘     └─────────────────────┘     └────────────────┘
//!                                  │                         │
//!                                  ▼                         ▼
//!                        ┌─────────────────┐        ┌────────────────┐
//!                        │   DLQ Stream    │        │   Processor    │
//!                        │ (Dead Letters)  │        │ (Your Logic)   │
//!                        └─────────────────┘        └────────────────┘
//! ```
//!
//! # Key Features
//!
//! - **JetStream Consumers**: Pull-based consumers with ack/nak semantics
//! - **Dead Letter Queue**: Failed messages moved to DLQ after max retries
//! - **Health Endpoints**: K8s-ready liveness/readiness probes
//! - **Prometheus Metrics**: Jobs processed, failed, latency histograms
//! - **Graceful Shutdown**: Drain in-flight messages before exit
//!
//! # Example
//!
//! ```rust,ignore
//! use nats_worker::{NatsWorker, NatsProducer, StreamConfig};
//! use messaging::{Job, Processor};
//!
//! // Define your stream
//! struct EmailStream;
//! impl StreamConfig for EmailStream {
//!     const STREAM_NAME: &'static str = "EMAILS";
//!     const CONSUMER_NAME: &'static str = "email-worker";
//!     const DLQ_STREAM: &'static str = "EMAILS_DLQ";
//! }
//!
//! // Create worker
//! let worker = NatsWorker::<EmailJob, EmailProcessor>::new(
//!     nats_client,
//!     processor,
//!     WorkerConfig::from_stream::<EmailStream>(),
//! ).await?;
//!
//! // Run with graceful shutdown
//! worker.run(shutdown_rx).await?;
//! ```
//!
//! # Comparison with stream-worker (Redis)
//!
//! | Aspect | nats-worker | stream-worker |
//! |--------|------------|---------------|
//! | Backend | NATS JetStream | Redis Streams |
//! | Consumer Model | Pull | Consumer Groups |
//! | Ack Model | AckSync/Nak | XACK |
//! | Redelivery | Built-in | XCLAIM |
//! | Pub/Sub | Excellent | Limited |
//! | Latency | ~100μs | ~1ms |

mod config;
mod consumer;
mod dlq;
mod error;
mod health;
pub mod metrics;
mod producer;
mod worker;

pub use config::{StreamConfig, WorkerConfig};
pub use consumer::NatsConsumer;
pub use dlq::DlqManager;
pub use error::NatsError;
pub use health::HealthServer;
pub use metrics::{init_metrics, NatsMetrics};
pub use producer::NatsProducer;
pub use worker::NatsWorker;

// Re-export from messaging
pub use messaging::{ErrorCategory, Job, JobEvent, ProcessingError, Processor};
