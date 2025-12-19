//! Stream Worker - Generic Redis Streams Worker Library
//!
//! A reusable library for building Redis Streams workers with:
//! - Consumer group support for horizontal scaling
//! - Automatic retry with exponential backoff and jitter
//! - Smart error categorization (transient, permanent, rate-limited)
//! - Dead letter queue (DLQ) for failed jobs with admin API
//! - Graceful shutdown handling
//! - Health check endpoints for Kubernetes probes
//! - Non-blocking polling for reliable operation with ConnectionManager
//! - Concurrent job processing with configurable parallelism
//! - **Circuit breaker** for cascading failure protection
//! - **Rate limiting** for external service calls
//!
//! # Architecture
//!
//! ```text
//! Redis Stream (domain:jobs)
//!   ↓ (Consumer Group)
//! StreamWorker<J, P>
//!   ↓ (processes jobs)
//! StreamProcessor<J>
//!   ↓ (on failure)
//! DLQ Stream (domain:dlq)
//! ```
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use stream_worker::{StreamJob, StreamProcessor, StreamWorker, WorkerConfig};
//!
//! // 1. Define your job type
//! #[derive(Clone, Serialize, Deserialize)]
//! struct MyJob {
//!     id: Uuid,
//!     payload: String,
//!     retry_count: u32,
//! }
//!
//! impl StreamJob for MyJob {
//!     fn job_id(&self) -> String { self.id.to_string() }
//!     fn retry_count(&self) -> u32 { self.retry_count }
//!     fn with_retry(&self) -> Self {
//!         Self { retry_count: self.retry_count + 1, ..self.clone() }
//!     }
//! }
//!
//! // 2. Define your processor
//! struct MyProcessor;
//!
//! #[async_trait]
//! impl StreamProcessor<MyJob> for MyProcessor {
//!     async fn process(&self, job: &MyJob) -> Result<(), StreamError> {
//!         // Process the job
//!         Ok(())
//!     }
//!     fn name(&self) -> &'static str { "MyProcessor" }
//! }
//!
//! // 3. Run the worker
//! let config = WorkerConfig::from_stream_def::<MyStreamDef>();
//! let worker = StreamWorker::new(redis, processor, config);
//! worker.run(shutdown_rx).await?;
//! ```

mod config;
mod consumer;
pub mod dlq;
mod error;
mod event;
mod health;
pub mod metrics;
mod producer;
mod registry;
pub mod resilience;
mod worker;

// Re-export all public types
pub use config::WorkerConfig;
pub use consumer::StreamConsumer;
pub use error::{ErrorCategory, RetryStrategy, StreamError};
pub use event::StreamEvent;
pub use health::{
    HealthState,
    health_handler,
    health_router,
    metrics_handler,
    ready_handler,
    stream_info_handler,
    // DLQ admin handlers
    dlq_admin_router,
    full_admin_router,
    dlq_stats_handler,
    dlq_list_handler,
    dlq_reprocess_one_handler,
    dlq_reprocess_batch_handler,
    dlq_archive_one_handler,
    dlq_archive_all_handler,
};
pub use producer::StreamProducer;
pub use registry::{Action, MessageKey, StreamDef, StreamId};
pub use resilience::{
    CircuitBreaker, CircuitBreakerConfig, CircuitState, RateLimiter, ResilienceError,
    ResilienceLayer,
};
pub use worker::{StreamJob, StreamProcessor, StreamWorker};

/// Result type alias for stream operations.
pub type StreamResult<T> = Result<T, StreamError>;
