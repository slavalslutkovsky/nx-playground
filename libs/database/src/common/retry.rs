use std::future::Future;
use std::time::Duration;
use tracing::{debug, warn};

/// Retry configuration for database connections
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,

    /// Initial delay between retries in milliseconds
    pub initial_delay_ms: u64,

    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,

    /// Multiplier for exponential backoff (typically 2.0)
    pub backoff_multiplier: f64,

    /// Whether to add jitter to prevent thundering herd
    pub use_jitter: bool,
}

impl RetryConfig {
    /// Create a new retry configuration with defaults
    ///
    /// Defaults:
    /// - max_retries: 3
    /// - initial_delay_ms: 100
    /// - max_delay_ms: 5000
    /// - backoff_multiplier: 2.0
    /// - use_jitter: true
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a retry config with custom max retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Create a retry config with custom initial delay
    pub fn with_initial_delay(mut self, delay_ms: u64) -> Self {
        self.initial_delay_ms = delay_ms;
        self
    }

    /// Create a retry config with custom max delay
    pub fn with_max_delay(mut self, delay_ms: u64) -> Self {
        self.max_delay_ms = delay_ms;
        self
    }

    /// Disable jitter
    pub fn without_jitter(mut self) -> Self {
        self.use_jitter = false;
        self
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
            use_jitter: true,
        }
    }
}

/// Retry an async operation with exponential backoff
///
/// # Arguments
/// * `operation` - The async operation to retry
/// * `config` - Retry configuration
///
/// # Example
/// ```ignore
/// use database::common::retry::{retry_with_backoff, RetryConfig};
///
/// let config = RetryConfig::new().with_max_retries(5);
///
/// let result = retry_with_backoff(
///     || async { database::postgres::connect(&db_url).await },
///     config
/// ).await?;
/// ```
pub async fn retry_with_backoff<F, Fut, T, E>(mut operation: F, config: RetryConfig) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay_ms;

    loop {
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    debug!("Operation succeeded after {} retries", attempt);
                }
                return Ok(result);
            }
            Err(e) => {
                attempt += 1;

                if attempt > config.max_retries {
                    warn!(
                        "Operation failed after {} attempts: {}",
                        config.max_retries, e
                    );
                    return Err(e);
                }

                // Calculate delay with exponential backoff
                let current_delay = if config.use_jitter {
                    apply_jitter(delay)
                } else {
                    delay
                };

                debug!(
                    "Operation failed (attempt {}/{}): {}. Retrying in {}ms...",
                    attempt, config.max_retries, e, current_delay
                );

                tokio::time::sleep(Duration::from_millis(current_delay)).await;

                // Exponential backoff for next iteration
                delay =
                    ((delay as f64 * config.backoff_multiplier) as u64).min(config.max_delay_ms);
            }
        }
    }
}

/// Apply jitter to a delay value to prevent thundering herd
///
/// Uses a random value between 50% and 100% of the original delay
fn apply_jitter(delay: u64) -> u64 {
    use std::collections::hash_map::RandomState;
    use std::hash::BuildHasher;

    // Use a simple pseudo-random approach based on current time
    let random_factor =
        (RandomState::new().hash_one(std::time::SystemTime::now()) % 50) as f64 / 100.0 + 0.5; // 0.5 to 1.0

    (delay as f64 * random_factor) as u64
}

/// Simplified retry with default configuration
///
/// Retries up to 3 times with exponential backoff starting at 100ms.
///
/// # Example
/// ```ignore
/// use database::common::retry::retry;
///
/// let result = retry(|| async {
///     database::postgres::connect(&db_url).await
/// }).await?;
/// ```
pub async fn retry<F, Fut, T, E>(operation: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    retry_with_backoff(operation, RetryConfig::default()).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry(|| {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok::<_, String>("success")
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let config = RetryConfig::new().with_initial_delay(10).without_jitter();

        let result = retry_with_backoff(
            || {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err(format!("Attempt {}", count + 1))
                    } else {
                        Ok("success")
                    }
                }
            },
            config,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_max_retries_exceeded() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let config = RetryConfig::new()
            .with_max_retries(2)
            .with_initial_delay(10)
            .without_jitter();

        let result = retry_with_backoff(
            || {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<String, _>("always fails")
                }
            },
            config,
        )
        .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "always fails");
        assert_eq!(counter.load(Ordering::SeqCst), 3); // 1 initial + 2 retries
    }

    #[tokio::test]
    async fn test_retry_config_builder() {
        let config = RetryConfig::new()
            .with_max_retries(5)
            .with_initial_delay(200)
            .with_max_delay(10000)
            .without_jitter();

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_delay_ms, 200);
        assert_eq!(config.max_delay_ms, 10000);
        assert!(!config.use_jitter);
    }

    #[test]
    fn test_apply_jitter() {
        let delay = 1000;
        for _ in 0..10 {
            let jittered = apply_jitter(delay);
            assert!(jittered >= 500); // At least 50%
            assert!(jittered <= 1000); // At most 100%
        }
    }

    #[tokio::test]
    async fn test_exponential_backoff() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        let start = std::time::Instant::now();

        let config = RetryConfig::new()
            .with_max_retries(3)
            .with_initial_delay(50)
            .without_jitter();

        let _result = retry_with_backoff(
            || {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<String, _>("fail")
                }
            },
            config,
        )
        .await;

        let elapsed = start.elapsed();
        // Should take at least 50 + 100 + 200 = 350ms
        assert!(elapsed.as_millis() >= 300);
    }
}
