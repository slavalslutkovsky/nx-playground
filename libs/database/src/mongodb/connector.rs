use mongodb::{Client, options::ClientOptions};
use std::time::Duration;
use tracing::info;

use super::MongoConfig;
use crate::common::{RetryConfig, retry, retry_with_backoff};

/// Error type for MongoDB operations
#[derive(Debug, thiserror::Error)]
pub enum MongoError {
    #[error("MongoDB error: {0}")]
    Mongo(#[from] mongodb::error::Error),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
}

/// Connect to MongoDB and return a Client
///
/// # Arguments
/// * `url` - MongoDB connection string (e.g., "mongodb://localhost:27017")
///
/// # Example
/// ```ignore
/// use database::mongodb::connect;
///
/// let client = connect("mongodb://localhost:27017").await?;
/// let db = client.database("mydb");
/// ```
pub async fn connect(url: &str) -> Result<Client, MongoError> {
    info!("Attempting to connect to MongoDB at {}", url);

    let mut options = ClientOptions::parse(url).await?;

    // Set reasonable defaults
    options.max_pool_size = Some(100);
    options.min_pool_size = Some(5);
    options.connect_timeout = Some(Duration::from_secs(10));
    options.server_selection_timeout = Some(Duration::from_secs(30));

    let client = Client::with_options(options)?;

    // Verify connection by listing databases (lightweight ping)
    client
        .list_database_names()
        .await
        .map_err(|e| MongoError::ConnectionFailed(e.to_string()))?;

    info!("Successfully connected to MongoDB");
    Ok(client)
}

/// Connect using a MongoConfig
///
/// This is the recommended way to connect when using configuration.
///
/// # Example
/// ```ignore
/// use database::mongodb::{MongoConfig, connect_from_config};
///
/// let config = MongoConfig::with_database("mongodb://localhost:27017", "mydb");
/// let client = connect_from_config(&config).await?;
/// ```
///
/// With FromEnv (requires `config` feature):
/// ```ignore
/// use database::mongodb::connect_from_config;
/// use core_config::FromEnv;
///
/// let config = MongoConfig::from_env()?;
/// let client = connect_from_config(&config).await?;
/// ```
pub async fn connect_from_config(config: &MongoConfig) -> Result<Client, MongoError> {
    info!("Attempting to connect to MongoDB at {}", config.url);

    let mut options = ClientOptions::parse(&config.url).await?;

    // Apply config settings
    options.max_pool_size = Some(config.max_pool_size);
    options.min_pool_size = Some(config.min_pool_size);
    options.connect_timeout = Some(Duration::from_secs(config.connect_timeout_secs));
    options.server_selection_timeout =
        Some(Duration::from_secs(config.server_selection_timeout_secs));

    if let Some(ref app_name) = config.app_name {
        options.app_name = Some(app_name.clone());
    }

    let client = Client::with_options(options)?;

    // Verify connection
    client
        .list_database_names()
        .await
        .map_err(|e| MongoError::ConnectionFailed(e.to_string()))?;

    info!("Successfully connected to MongoDB");
    Ok(client)
}

/// Connect to MongoDB with automatic retry on failure
///
/// Uses exponential backoff with jitter to retry connection attempts.
/// Useful for handling transient network issues during startup.
///
/// # Example
/// ```ignore
/// use database::mongodb::connect_with_retry;
/// use database::common::RetryConfig;
///
/// // Default retry: 3 attempts, 100ms initial delay
/// let client = connect_with_retry("mongodb://localhost:27017", None).await?;
///
/// // Custom retry: 5 attempts, 500ms initial delay
/// let config = RetryConfig::new()
///     .with_max_retries(5)
///     .with_initial_delay(500);
/// let client = connect_with_retry("mongodb://localhost:27017", Some(config)).await?;
/// ```
pub async fn connect_with_retry(
    url: &str,
    retry_config: Option<RetryConfig>,
) -> Result<Client, MongoError> {
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
/// use database::mongodb::{MongoConfig, connect_from_config_with_retry};
/// use database::common::RetryConfig;
///
/// let config = MongoConfig::from_env()?;
/// let retry_config = RetryConfig::new().with_max_retries(5);
/// let client = connect_from_config_with_retry(&config, Some(retry_config)).await?;
/// ```
pub async fn connect_from_config_with_retry(
    config: &MongoConfig,
    retry_config: Option<RetryConfig>,
) -> Result<Client, MongoError> {
    let config_clone = config.clone();

    match retry_config {
        Some(retry) => retry_with_backoff(|| connect_from_config(&config_clone), retry).await,
        None => retry(|| connect_from_config(&config_clone)).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires actual MongoDB
    async fn test_connect() {
        let mongo_url = std::env::var("MONGODB_URL")
            .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

        let result = connect(&mongo_url).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires actual MongoDB
    async fn test_connect_from_config() {
        let config = MongoConfig::with_database("mongodb://localhost:27017", "test");
        let result = connect_from_config(&config).await;
        assert!(result.is_ok());
    }
}
