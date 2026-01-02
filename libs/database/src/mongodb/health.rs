use mongodb::Client;
use std::time::Instant;

/// Health check status for MongoDB
#[derive(Debug, Clone)]
pub struct HealthStatus {
    /// Whether the database is healthy
    pub healthy: bool,
    /// Optional message (e.g., error details)
    pub message: Option<String>,
    /// Response time in milliseconds
    pub response_time_ms: u64,
}

/// Check MongoDB health with a simple ping command
///
/// # Arguments
/// * `client` - MongoDB client
///
/// # Example
/// ```ignore
/// use database::mongodb::{connect, check_health};
///
/// let client = connect("mongodb://localhost:27017").await?;
/// let healthy = check_health(&client).await;
/// ```
pub async fn check_health(client: &Client) -> bool {
    // Run a lightweight command to check connectivity
    client.list_database_names().await.is_ok()
}

/// Check MongoDB health with detailed status
///
/// Returns timing information and any error messages.
///
/// # Arguments
/// * `client` - MongoDB client
///
/// # Example
/// ```ignore
/// use database::mongodb::{connect, check_health_detailed};
///
/// let client = connect("mongodb://localhost:27017").await?;
/// let status = check_health_detailed(&client).await;
/// if status.healthy {
///     println!("MongoDB healthy, latency: {}ms", status.response_time_ms);
/// } else {
///     println!("MongoDB unhealthy: {:?}", status.message);
/// }
/// ```
pub async fn check_health_detailed(client: &Client) -> HealthStatus {
    let start = Instant::now();

    match client.list_database_names().await {
        Ok(_) => {
            let elapsed = start.elapsed();
            HealthStatus {
                healthy: true,
                message: None,
                response_time_ms: elapsed.as_millis() as u64,
            }
        }
        Err(e) => {
            let elapsed = start.elapsed();
            HealthStatus {
                healthy: false,
                message: Some(e.to_string()),
                response_time_ms: elapsed.as_millis() as u64,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires actual MongoDB
    async fn test_check_health() {
        let client = Client::with_uri_str("mongodb://localhost:27017")
            .await
            .unwrap();
        let healthy = check_health(&client).await;
        assert!(healthy);
    }

    #[tokio::test]
    #[ignore] // Requires actual MongoDB
    async fn test_check_health_detailed() {
        let client = Client::with_uri_str("mongodb://localhost:27017")
            .await
            .unwrap();
        let status = check_health_detailed(&client).await;
        assert!(status.healthy);
        assert!(status.message.is_none());
        assert!(status.response_time_ms > 0);
    }
}
