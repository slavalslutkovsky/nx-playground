use scylla::client::session::Session;
use scylla::client::session_builder::SessionBuilder;
use scylla::errors::{ExecutionError, NewSessionError};
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

use super::CassandraConfig;
use crate::common::{RetryConfig, retry, retry_with_backoff};

/// Error type for Cassandra operations
#[derive(Debug, thiserror::Error)]
pub enum CassandraError {
    #[error("Cassandra error: {0}")]
    Scylla(#[from] NewSessionError),

    #[error("Execution error: {0}")]
    Execution(#[from] ExecutionError),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Keyspace error: {0}")]
    KeyspaceError(String),
}

/// Cassandra session wrapper for connection pooling
pub type CassandraSession = Arc<Session>;

/// Connect to Cassandra/ScyllaDB and return a Session
///
/// # Arguments
/// * `contact_points` - List of Cassandra nodes (e.g., ["127.0.0.1:9042"])
///
/// # Example
/// ```ignore
/// use database::cassandra::connect;
///
/// let session = connect(&["127.0.0.1:9042"]).await?;
/// session.query_unpaged("SELECT * FROM system.local", &[]).await?;
/// ```
pub async fn connect(
    contact_points: &[impl AsRef<str>],
) -> Result<CassandraSession, CassandraError> {
    let points: Vec<&str> = contact_points.iter().map(|s| s.as_ref()).collect();
    info!("Attempting to connect to Cassandra at {:?}", points);

    let session: Session = SessionBuilder::new()
        .known_nodes(&points)
        .connection_timeout(Duration::from_secs(10))
        .build()
        .await?;

    // Verify connection by querying system table
    session
        .query_unpaged("SELECT release_version FROM system.local", &[])
        .await
        .map_err(|e| CassandraError::ConnectionFailed(e.to_string()))?;

    info!("Successfully connected to Cassandra");
    Ok(Arc::new(session))
}

/// Connect using a CassandraConfig
///
/// This is the recommended way to connect when using configuration.
///
/// # Example
/// ```ignore
/// use database::cassandra::{CassandraConfig, connect_from_config};
///
/// let config = CassandraConfig::with_keyspace(vec!["127.0.0.1:9042"], "mykeyspace");
/// let session = connect_from_config(&config).await?;
/// ```
///
/// With FromEnv (requires `config` feature):
/// ```ignore
/// use database::cassandra::connect_from_config;
/// use core_config::FromEnv;
///
/// let config = CassandraConfig::from_env()?;
/// let session = connect_from_config(&config).await?;
/// ```
pub async fn connect_from_config(
    config: &CassandraConfig,
) -> Result<CassandraSession, CassandraError> {
    info!(
        "Attempting to connect to Cassandra at {:?}",
        config.contact_points
    );

    let points: Vec<&str> = config.contact_points.iter().map(|s| s.as_str()).collect();

    let mut builder = SessionBuilder::new()
        .known_nodes(&points)
        .connection_timeout(Duration::from_secs(config.connect_timeout_secs));

    // Set authentication if provided
    if let (Some(username), Some(password)) = (&config.username, &config.password) {
        builder = builder.user(username, password);
    }

    // Set default keyspace if provided
    if let Some(ref keyspace) = config.keyspace {
        builder = builder.use_keyspace(keyspace, true);
    }

    let session: Session = builder.build().await?;

    // Verify connection
    session
        .query_unpaged("SELECT release_version FROM system.local", &[])
        .await
        .map_err(|e| CassandraError::ConnectionFailed(e.to_string()))?;

    info!("Successfully connected to Cassandra");
    Ok(Arc::new(session))
}

/// Connect to Cassandra with automatic retry on failure
///
/// Uses exponential backoff with jitter to retry connection attempts.
/// Useful for handling transient network issues during startup.
///
/// # Example
/// ```ignore
/// use database::cassandra::connect_with_retry;
/// use database::common::RetryConfig;
///
/// // Default retry: 3 attempts, 100ms initial delay
/// let session = connect_with_retry(&["127.0.0.1:9042"], None).await?;
///
/// // Custom retry: 5 attempts, 500ms initial delay
/// let config = RetryConfig::new()
///     .with_max_retries(5)
///     .with_initial_delay(500);
/// let session = connect_with_retry(&["127.0.0.1:9042"], Some(config)).await?;
/// ```
pub async fn connect_with_retry(
    contact_points: &[impl AsRef<str> + Clone],
    retry_config: Option<RetryConfig>,
) -> Result<CassandraSession, CassandraError> {
    let points: Vec<String> = contact_points
        .iter()
        .map(|s| s.as_ref().to_string())
        .collect();

    match retry_config {
        Some(config) => {
            retry_with_backoff(
                || {
                    let p = points.clone();
                    async move { connect(&p).await }
                },
                config,
            )
            .await
        }
        None => {
            retry(|| {
                let p = points.clone();
                async move { connect(&p).await }
            })
            .await
        }
    }
}

/// Connect from config with automatic retry on failure
///
/// # Example
/// ```ignore
/// use database::cassandra::{CassandraConfig, connect_from_config_with_retry};
/// use database::common::RetryConfig;
///
/// let config = CassandraConfig::from_env()?;
/// let retry_config = RetryConfig::new().with_max_retries(5);
/// let session = connect_from_config_with_retry(&config, Some(retry_config)).await?;
/// ```
pub async fn connect_from_config_with_retry(
    config: &CassandraConfig,
    retry_config: Option<RetryConfig>,
) -> Result<CassandraSession, CassandraError> {
    let config_clone = config.clone();

    match retry_config {
        Some(retry) => retry_with_backoff(|| connect_from_config(&config_clone), retry).await,
        None => retry(|| connect_from_config(&config_clone)).await,
    }
}

/// Create a keyspace if it doesn't exist
///
/// # Example
/// ```ignore
/// use database::cassandra::{connect, create_keyspace_if_not_exists};
///
/// let session = connect(&["127.0.0.1:9042"]).await?;
/// create_keyspace_if_not_exists(&session, "mykeyspace", 1).await?;
/// ```
pub async fn create_keyspace_if_not_exists(
    session: &Session,
    keyspace: &str,
    replication_factor: u32,
) -> Result<(), CassandraError> {
    let query = format!(
        "CREATE KEYSPACE IF NOT EXISTS {} WITH replication = {{'class': 'SimpleStrategy', 'replication_factor': {}}}",
        keyspace, replication_factor
    );

    session
        .query_unpaged(query, &[])
        .await
        .map_err(|e| CassandraError::KeyspaceError(e.to_string()))?;

    info!("Keyspace '{}' ready", keyspace);
    Ok(())
}

/// Use a specific keyspace
///
/// # Example
/// ```ignore
/// use database::cassandra::{connect, use_keyspace};
///
/// let session = connect(&["127.0.0.1:9042"]).await?;
/// use_keyspace(&session, "mykeyspace").await?;
/// ```
pub async fn use_keyspace(session: &Session, keyspace: &str) -> Result<(), CassandraError> {
    session
        .use_keyspace(keyspace, true)
        .await
        .map_err(|e| CassandraError::KeyspaceError(e.to_string()))?;

    info!("Using keyspace '{}'", keyspace);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires actual Cassandra
    async fn test_connect() {
        let contact_points = std::env::var("CASSANDRA_CONTACT_POINTS")
            .unwrap_or_else(|_| "127.0.0.1:9042".to_string());
        let points: Vec<&str> = contact_points.split(',').collect();

        let result = connect(&points).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires actual Cassandra
    async fn test_connect_from_config() {
        let config = CassandraConfig::new(vec!["127.0.0.1:9042"]);
        let result = connect_from_config(&config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires actual Cassandra
    async fn test_create_keyspace() {
        let session = connect(&["127.0.0.1:9042"]).await.unwrap();
        let result = create_keyspace_if_not_exists(&session, "test_keyspace", 1).await;
        assert!(result.is_ok());
    }
}
