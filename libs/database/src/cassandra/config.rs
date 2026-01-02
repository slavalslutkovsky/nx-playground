#[cfg(feature = "config")]
use core_config::{ConfigError, FromEnv};

/// Cassandra/ScyllaDB database configuration
///
/// This struct holds Cassandra connection settings.
/// It can be constructed manually or loaded from environment variables (with `config` feature).
///
/// # Example
///
/// ```ignore
/// use database::cassandra::CassandraConfig;
///
/// // Manual construction
/// let config = CassandraConfig::new(vec!["127.0.0.1:9042"]);
///
/// // With keyspace
/// let config = CassandraConfig::with_keyspace(vec!["127.0.0.1:9042"], "mykeyspace");
///
/// // From environment variables (requires `config` feature)
/// let config = CassandraConfig::from_env()?;
/// ```
#[derive(Clone, Debug)]
pub struct CassandraConfig {
    /// Contact points (host:port pairs)
    /// Example: ["127.0.0.1:9042", "127.0.0.2:9042"]
    pub contact_points: Vec<String>,

    /// Keyspace to use (similar to a database in SQL)
    pub keyspace: Option<String>,

    /// Optional datacenter for DC-aware load balancing
    pub local_datacenter: Option<String>,

    /// Optional username for authentication
    pub username: Option<String>,

    /// Optional password for authentication
    pub password: Option<String>,

    /// Connection timeout in seconds
    pub connect_timeout_secs: u64,

    /// Request timeout in seconds
    pub request_timeout_secs: u64,

    /// Enable SSL/TLS
    pub ssl_enabled: bool,

    /// Number of connections per host
    pub connections_per_host: usize,
}

impl CassandraConfig {
    /// Create a new CassandraConfig with contact points
    ///
    /// # Arguments
    /// * `contact_points` - List of Cassandra nodes (e.g., ["127.0.0.1:9042"])
    ///
    /// # Example
    /// ```ignore
    /// let config = CassandraConfig::new(vec!["127.0.0.1:9042"]);
    /// ```
    pub fn new<S: Into<String>>(contact_points: Vec<S>) -> Self {
        Self {
            contact_points: contact_points.into_iter().map(|s| s.into()).collect(),
            keyspace: None,
            local_datacenter: None,
            username: None,
            password: None,
            connect_timeout_secs: 10,
            request_timeout_secs: 30,
            ssl_enabled: false,
            connections_per_host: 1,
        }
    }

    /// Create a CassandraConfig with a specific keyspace
    ///
    /// # Example
    /// ```ignore
    /// let config = CassandraConfig::with_keyspace(
    ///     vec!["127.0.0.1:9042"],
    ///     "mykeyspace"
    /// );
    /// ```
    pub fn with_keyspace<S: Into<String>>(
        contact_points: Vec<S>,
        keyspace: impl Into<String>,
    ) -> Self {
        Self {
            contact_points: contact_points.into_iter().map(|s| s.into()).collect(),
            keyspace: Some(keyspace.into()),
            local_datacenter: None,
            username: None,
            password: None,
            connect_timeout_secs: 10,
            request_timeout_secs: 30,
            ssl_enabled: false,
            connections_per_host: 1,
        }
    }

    /// Set the local datacenter for DC-aware load balancing
    pub fn with_datacenter(mut self, datacenter: impl Into<String>) -> Self {
        self.local_datacenter = Some(datacenter.into());
        self
    }

    /// Set authentication credentials
    pub fn with_credentials(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    /// Enable SSL/TLS
    pub fn with_ssl(mut self, enabled: bool) -> Self {
        self.ssl_enabled = enabled;
        self
    }

    /// Set connection timeout
    pub fn with_connect_timeout(mut self, secs: u64) -> Self {
        self.connect_timeout_secs = secs;
        self
    }

    /// Set request timeout
    pub fn with_request_timeout(mut self, secs: u64) -> Self {
        self.request_timeout_secs = secs;
        self
    }

    /// Set connections per host
    pub fn with_connections_per_host(mut self, count: usize) -> Self {
        self.connections_per_host = count;
        self
    }

    /// Get the contact points
    pub fn contact_points(&self) -> &[String] {
        &self.contact_points
    }

    /// Get the keyspace
    pub fn keyspace(&self) -> Option<&str> {
        self.keyspace.as_deref()
    }
}

impl Default for CassandraConfig {
    fn default() -> Self {
        Self {
            contact_points: vec!["127.0.0.1:9042".to_string()],
            keyspace: None,
            local_datacenter: None,
            username: None,
            password: None,
            connect_timeout_secs: 10,
            request_timeout_secs: 30,
            ssl_enabled: false,
            connections_per_host: 1,
        }
    }
}

/// Load CassandraConfig from environment variables
///
/// Environment variables:
/// - `CASSANDRA_CONTACT_POINTS` (required) - Comma-separated list of contact points
///   Example: "127.0.0.1:9042,127.0.0.2:9042"
/// - `CASSANDRA_KEYSPACE` (optional) - Keyspace name
/// - `CASSANDRA_DATACENTER` (optional) - Local datacenter for load balancing
/// - `CASSANDRA_USERNAME` (optional) - Authentication username
/// - `CASSANDRA_PASSWORD` (optional) - Authentication password
/// - `CASSANDRA_CONNECT_TIMEOUT_SECS` (optional, default: 10)
/// - `CASSANDRA_REQUEST_TIMEOUT_SECS` (optional, default: 30)
/// - `CASSANDRA_SSL_ENABLED` (optional, default: false)
/// - `CASSANDRA_CONNECTIONS_PER_HOST` (optional, default: 1)
///
/// # Example
/// ```ignore
/// use database::cassandra::CassandraConfig;
/// use core_config::FromEnv;
///
/// let config = CassandraConfig::from_env()?;
/// ```
#[cfg(feature = "config")]
impl FromEnv for CassandraConfig {
    fn from_env() -> Result<Self, ConfigError> {
        let contact_points_str = std::env::var("CASSANDRA_CONTACT_POINTS")
            .map_err(|_| ConfigError::MissingEnvVar("CASSANDRA_CONTACT_POINTS".to_string()))?;

        let contact_points: Vec<String> = contact_points_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if contact_points.is_empty() {
            return Err(ConfigError::ParseError {
                key: "CASSANDRA_CONTACT_POINTS".to_string(),
                details: "No valid contact points provided".to_string(),
            });
        }

        let keyspace = std::env::var("CASSANDRA_KEYSPACE").ok();
        let local_datacenter = std::env::var("CASSANDRA_DATACENTER").ok();
        let username = std::env::var("CASSANDRA_USERNAME").ok();
        let password = std::env::var("CASSANDRA_PASSWORD").ok();

        let connect_timeout_secs = std::env::var("CASSANDRA_CONNECT_TIMEOUT_SECS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .map_err(|e| ConfigError::ParseError {
                key: "CASSANDRA_CONNECT_TIMEOUT_SECS".to_string(),
                details: format!("{}", e),
            })?;

        let request_timeout_secs = std::env::var("CASSANDRA_REQUEST_TIMEOUT_SECS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .map_err(|e| ConfigError::ParseError {
                key: "CASSANDRA_REQUEST_TIMEOUT_SECS".to_string(),
                details: format!("{}", e),
            })?;

        let ssl_enabled = std::env::var("CASSANDRA_SSL_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .map_err(|e| ConfigError::ParseError {
                key: "CASSANDRA_SSL_ENABLED".to_string(),
                details: format!("{}", e),
            })?;

        let connections_per_host = std::env::var("CASSANDRA_CONNECTIONS_PER_HOST")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .map_err(|e| ConfigError::ParseError {
                key: "CASSANDRA_CONNECTIONS_PER_HOST".to_string(),
                details: format!("{}", e),
            })?;

        Ok(Self {
            contact_points,
            keyspace,
            local_datacenter,
            username,
            password,
            connect_timeout_secs,
            request_timeout_secs,
            ssl_enabled,
            connections_per_host,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cassandra_config_new() {
        let config = CassandraConfig::new(vec!["127.0.0.1:9042"]);
        assert_eq!(config.contact_points, vec!["127.0.0.1:9042"]);
        assert!(config.keyspace.is_none());
        assert_eq!(config.connect_timeout_secs, 10);
    }

    #[test]
    fn test_cassandra_config_with_keyspace() {
        let config = CassandraConfig::with_keyspace(vec!["127.0.0.1:9042"], "mykeyspace");
        assert_eq!(config.contact_points, vec!["127.0.0.1:9042"]);
        assert_eq!(config.keyspace, Some("mykeyspace".to_string()));
    }

    #[test]
    fn test_cassandra_config_builder_pattern() {
        let config = CassandraConfig::new(vec!["127.0.0.1:9042"])
            .with_datacenter("dc1")
            .with_credentials("user", "pass")
            .with_ssl(true)
            .with_connect_timeout(30);

        assert_eq!(config.local_datacenter, Some("dc1".to_string()));
        assert_eq!(config.username, Some("user".to_string()));
        assert_eq!(config.password, Some("pass".to_string()));
        assert!(config.ssl_enabled);
        assert_eq!(config.connect_timeout_secs, 30);
    }

    #[test]
    fn test_cassandra_config_default() {
        let config = CassandraConfig::default();
        assert_eq!(config.contact_points, vec!["127.0.0.1:9042"]);
        assert!(config.keyspace.is_none());
    }

    #[cfg(feature = "config")]
    #[test]
    fn test_cassandra_config_from_env() {
        temp_env::with_vars(
            [
                (
                    "CASSANDRA_CONTACT_POINTS",
                    Some("127.0.0.1:9042,127.0.0.2:9042"),
                ),
                ("CASSANDRA_KEYSPACE", Some("testkeyspace")),
            ],
            || {
                let config = CassandraConfig::from_env();
                assert!(config.is_ok());
                let config = config.unwrap();
                assert_eq!(config.contact_points.len(), 2);
                assert_eq!(config.keyspace, Some("testkeyspace".to_string()));
            },
        );
    }

    #[cfg(feature = "config")]
    #[test]
    fn test_cassandra_config_from_env_missing() {
        temp_env::with_vars([("CASSANDRA_CONTACT_POINTS", None::<&str>)], || {
            let config = CassandraConfig::from_env();
            assert!(config.is_err());
        });
    }
}
