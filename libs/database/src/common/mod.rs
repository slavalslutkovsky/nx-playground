//! Common utilities shared across all database implementations

pub mod error;
pub mod retry;

pub use error::{DatabaseError, DatabaseResult};
pub use retry::{RetryConfig, retry, retry_with_backoff};
