/// Re-export tonic's Interceptor trait for convenience
pub use tonic::service::Interceptor;

pub mod auth;
pub mod tracing;
pub mod metrics;
pub mod compose;

pub use auth::AuthInterceptor;
pub use tracing::TracingInterceptor;
pub use metrics::MetricsInterceptor;
pub use compose::{compose_interceptors, ComposedInterceptor};
