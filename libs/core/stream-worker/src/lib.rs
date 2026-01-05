//! Stream Worker Framework
//!
//! A generic Redis Streams worker framework for processing background jobs.
//!
//! ## Features
//!
//! - **Generic worker**: `StreamWorker<J, P>` processes any job type
//! - **Consumer groups**: Horizontal scaling with Redis consumer groups
//! - **Dead Letter Queue**: Failed jobs moved to DLQ after max retries
//! - **Prometheus metrics**: Built-in observability
//! - **Circuit breaker**: Protect downstream services
//! - **Health endpoints**: K8s-ready liveness and readiness probes
//!
//! ## Example
//!
//! ```ignore
//! use stream_worker::{StreamWorker, StreamJob, StreamProcessor, StreamDef, WorkerConfig};
//!
//! // Define your job type
//! #[derive(Clone, Serialize, Deserialize)]
//! struct MyJob { /* ... */ }
//!
//! impl StreamJob for MyJob { /* ... */ }
//!
//! // Define your stream
//! struct MyStream;
//! impl StreamDef for MyStream {
//!     const STREAM_NAME: &'static str = "my:jobs";
//!     const CONSUMER_GROUP: &'static str = "my_workers";
//!     const DLQ_STREAM: &'static str = "my:dlq";
//! }
//!
//! // Create processor and run
//! let config = WorkerConfig::from_stream_def::<MyStream>();
//! let worker = StreamWorker::new(redis, processor, config);
//! worker.run(shutdown_rx).await?;
//! ```

mod config;
mod consumer;
mod dlq;
mod error;
mod event;
mod health;
pub mod metrics;
mod producer;
mod registry;
mod resilience;
mod worker;

// Re-export main types
pub use config::WorkerConfig;
pub use consumer::StreamConsumer;
pub use dlq::DlqManager;
pub use error::{ErrorCategory, StreamError};
pub use event::StreamEvent;
pub use health::{full_admin_router, health_router, HealthState};
pub use metrics::{init_metrics, StreamMetrics};
pub use producer::StreamProducer;
pub use registry::{StreamDef, StreamJob, StreamProcessor};
pub use resilience::{CircuitBreaker, CircuitState, RateLimiter};
pub use worker::StreamWorker;
