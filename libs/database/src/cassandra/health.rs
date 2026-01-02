use scylla::client::session::Session;
use scylla::response::query_result::QueryResult;
use std::time::Instant;

/// Health check status for Cassandra
#[derive(Debug, Clone)]
pub struct HealthStatus {
    /// Whether the database is healthy
    pub healthy: bool,
    /// Optional message (e.g., error details)
    pub message: Option<String>,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Cassandra release version (if available)
    pub version: Option<String>,
}

/// Check Cassandra health with a simple query
///
/// # Arguments
/// * `session` - Cassandra session
///
/// # Example
/// ```ignore
/// use database::cassandra::{connect, check_health};
///
/// let session = connect(&["127.0.0.1:9042"]).await?;
/// let healthy = check_health(&session).await;
/// ```
pub async fn check_health(session: &Session) -> bool {
    session
        .query_unpaged("SELECT release_version FROM system.local", &[])
        .await
        .is_ok()
}

/// Check Cassandra health with detailed status
///
/// Returns timing information, version, and any error messages.
///
/// # Arguments
/// * `session` - Cassandra session
///
/// # Example
/// ```ignore
/// use database::cassandra::{connect, check_health_detailed};
///
/// let session = connect(&["127.0.0.1:9042"]).await?;
/// let status = check_health_detailed(&session).await;
/// if status.healthy {
///     println!("Cassandra healthy, version: {:?}, latency: {}ms",
///         status.version, status.response_time_ms);
/// } else {
///     println!("Cassandra unhealthy: {:?}", status.message);
/// }
/// ```
pub async fn check_health_detailed(session: &Session) -> HealthStatus {
    let start = Instant::now();

    match session
        .query_unpaged("SELECT release_version FROM system.local", &[])
        .await
    {
        Ok(result) => {
            let elapsed = start.elapsed();

            // Try to extract a version from a result
            let version = extract_version(result);

            HealthStatus {
                healthy: true,
                message: None,
                response_time_ms: elapsed.as_millis() as u64,
                version,
            }
        }
        Err(e) => {
            let elapsed = start.elapsed();
            HealthStatus {
                healthy: false,
                message: Some(e.to_string()),
                response_time_ms: elapsed.as_millis() as u64,
                version: None,
            }
        }
    }
}

fn extract_version(result: QueryResult) -> Option<String> {
    let rows_result = result.into_rows_result().ok()?;
    let mut rows = rows_result.rows::<(String,)>().ok()?;
    let row: Result<(String,), _> = rows.next()?;
    row.ok().map(|(v,)| v)
}

/// Get cluster information
///
/// Returns information about the Cassandra cluster nodes.
///
/// # Example
/// ```ignore
/// use database::cassandra::{connect, get_cluster_info};
///
/// let session = connect(&["127.0.0.1:9042"]).await?;
/// let info = get_cluster_info(&session).await?;
/// println!("Cluster: {:?}, Datacenter: {:?}", info.cluster_name, info.datacenter);
/// ```
#[derive(Debug, Clone)]
pub struct ClusterInfo {
    pub cluster_name: Option<String>,
    pub datacenter: Option<String>,
    pub rack: Option<String>,
    pub release_version: Option<String>,
}

pub async fn get_cluster_info(
    session: &Session,
) -> Result<ClusterInfo, super::connector::CassandraError> {
    let result = session
        .query_unpaged(
            "SELECT cluster_name, data_center, rack, release_version FROM system.local",
            &[],
        )
        .await?;

    let mut info = ClusterInfo {
        cluster_name: None,
        datacenter: None,
        rack: None,
        release_version: None,
    };

    if let Ok(rows_result) = result.into_rows_result()
        && let Ok(mut rows) = rows_result.rows::<(
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        )>()
        && let Some(Ok((cluster_name, datacenter, rack, release_version))) = rows.next()
    {
        info.cluster_name = cluster_name;
        info.datacenter = datacenter;
        info.rack = rack;
        info.release_version = release_version;
    }

    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use scylla::client::session_builder::SessionBuilder;

    #[tokio::test]
    #[ignore] // Requires actual Cassandra
    async fn test_check_health() {
        let session: Session = SessionBuilder::new()
            .known_node("127.0.0.1:9042")
            .build()
            .await
            .unwrap();

        let healthy = check_health(&session).await;
        assert!(healthy);
    }

    #[tokio::test]
    #[ignore] // Requires actual Cassandra
    async fn test_check_health_detailed() {
        let session: Session = SessionBuilder::new()
            .known_node("127.0.0.1:9042")
            .build()
            .await
            .unwrap();

        let status = check_health_detailed(&session).await;
        assert!(status.healthy);
        assert!(status.message.is_none());
        assert!(status.response_time_ms > 0);
    }

    #[tokio::test]
    #[ignore] // Requires actual Cassandra
    async fn test_get_cluster_info() {
        let session: Session = SessionBuilder::new()
            .known_node("127.0.0.1:9042")
            .build()
            .await
            .unwrap();

        let info = get_cluster_info(&session).await;
        assert!(info.is_ok());
    }
}
