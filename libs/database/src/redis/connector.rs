use redis::Client;
use redis::aio::ConnectionManager;
use tracing::info;

use super::RedisConfig;
use crate::common::{RetryConfig, retry, retry_with_backoff};

/// Connect to Redis and return a ConnectionManager
///
/// The ConnectionManager automatically handles connection failures and reconnections.
///
/// # Arguments
/// * `url` - Redis connection string (e.g., "redis://127.0.0.1:6379")
///
/// # Example
/// ```ignore
/// use database::redis::connect;
/// use redis::AsyncCommands;
///
/// let mut conn = connect("redis://127.0.0.1:6379").await?;
/// conn.set::<_, _, ()>("key", "value").await?;
/// ```
pub async fn connect(url: &str) -> redis::RedisResult<ConnectionManager> {
    info!("Attempting to connect to Redis at {}", url);

    let client = Client::open(url)?;
    let manager = ConnectionManager::new(client).await?;

    // Verify connection with PING
    let mut conn = manager.clone();
    let _: String = redis::cmd("PING").query_async(&mut conn).await?;

    info!("Successfully connected to Redis");
    Ok(manager)
}

/// Connect using a RedisConfig
///
/// This is the recommended way to connect when using configuration.
///
/// # Example
/// ```ignore
/// use database::redis::{RedisConfig, connect_from_config};
///
/// let config = RedisConfig::new("redis://127.0.0.1:6379");
/// let conn = connect_from_config(config).await?;
/// ```
///
/// With FromEnv (requires `config` feature):
/// ```ignore
/// use database::redis::connect_from_config;
/// use core_config::FromEnv;
///
/// let config = RedisConfig::from_env()?;
/// let conn = connect_from_config(config).await?;
/// ```
pub async fn connect_from_config(config: RedisConfig) -> redis::RedisResult<ConnectionManager> {
    connect(&config.url).await
}

/// Connect to Redis with automatic retry on failure
///
/// Uses exponential backoff with jitter to retry connection attempts.
/// Useful for handling transient network issues during startup.
///
/// # Example
/// ```ignore
/// use database::redis::connect_with_retry;
/// use database::common::RetryConfig;
///
/// // Default retry: 3 attempts, 100ms initial delay
/// let conn = connect_with_retry("redis://127.0.0.1:6379", None).await?;
///
/// // Custom retry: 5 attempts, 500ms initial delay
/// let config = RetryConfig::new()
///     .with_max_retries(5)
///     .with_initial_delay(500);
/// let conn = connect_with_retry("redis://127.0.0.1:6379", Some(config)).await?;
/// ```
pub async fn connect_with_retry(
    url: &str,
    retry_config: Option<RetryConfig>,
) -> redis::RedisResult<ConnectionManager> {
    let url_owned = url.to_string();

    match retry_config {
        Some(config) => retry_with_backoff(|| connect(&url_owned), config).await,
        None => retry(|| connect(&url_owned)).await,
    }
}

/// Connect from config with automatic retry on failure
///
/// # Example
/// ```ignore
/// use database::redis::{RedisConfig, connect_from_config_with_retry};
/// use database::common::RetryConfig;
///
/// let config = RedisConfig::from_env()?;
/// let retry_config = RetryConfig::new().with_max_retries(5);
/// let conn = connect_from_config_with_retry(config, Some(retry_config)).await?;
/// ```
pub async fn connect_from_config_with_retry(
    config: RedisConfig,
    retry_config: Option<RetryConfig>,
) -> redis::RedisResult<ConnectionManager> {
    connect_with_retry(&config.url, retry_config).await
}

/// Simple wrapper for Redis ConnectionManager (kept for compatibility)
///
/// Note: This is largely redundant with ConnectionManager itself.
/// Consider using `connect()` directly instead.
#[derive(Clone)]
pub struct RedisConnector {
    manager: ConnectionManager,
}

impl RedisConnector {
    /// Create a new RedisConnector from a connection URL
    ///
    /// # Example
    /// ```ignore
    /// use database::redis::RedisConnector;
    ///
    /// let connector = RedisConnector::new("redis://127.0.0.1:6379").await?;
    /// let conn = connector.manager();
    /// ```
    pub async fn new(url: &str) -> redis::RedisResult<Self> {
        let manager = connect(url).await?;
        Ok(Self { manager })
    }

    /// Get a cloned ConnectionManager
    pub fn manager(&self) -> ConnectionManager {
        self.manager.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires actual Redis
    async fn test_connect() {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        let result = connect(&redis_url).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires actual Redis
    async fn test_redis_connector() {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        let connector = RedisConnector::new(&redis_url).await;
        assert!(connector.is_ok());

        let connector = connector.unwrap();
        let _manager = connector.manager();
    }
}
