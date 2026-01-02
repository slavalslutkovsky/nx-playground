#[cfg(feature = "config")]
use core_config::{ConfigError, FromEnv};

/// MongoDB database configuration
///
/// This struct holds MongoDB connection settings.
/// It can be constructed manually or loaded from environment variables (with `config` feature).
///
/// # Example
///
/// ```ignore
/// use database::mongodb::MongoConfig;
///
/// // Manual construction
/// let config = MongoConfig::new("mongodb://localhost:27017");
///
/// // With database name
/// let config = MongoConfig::with_database("mongodb://localhost:27017", "mydb");
///
/// // From environment variables (requires `config` feature)
/// let config = MongoConfig::from_env()?;
/// ```
#[derive(Clone, Debug)]
pub struct MongoConfig {
    /// MongoDB connection URL (required)
    /// Format: mongodb://[username:password@]host[:port][/database][?options]
    pub url: String,

    /// Database name to use
    pub database: String,

    /// Optional application name for server logs
    pub app_name: Option<String>,

    /// Maximum number of connections in the pool
    pub max_pool_size: u32,

    /// Minimum number of connections in the pool
    pub min_pool_size: u32,

    /// Connection timeout in seconds
    pub connect_timeout_secs: u64,

    /// Server selection timeout in seconds
    pub server_selection_timeout_secs: u64,
}

impl MongoConfig {
    /// Create a new MongoConfig with just a URL and default database
    ///
    /// # Arguments
    /// * `url` - MongoDB connection string (e.g., "mongodb://localhost:27017")
    ///
    /// # Example
    /// ```ignore
    /// let config = MongoConfig::new("mongodb://localhost:27017");
    /// ```
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            database: "default".to_string(),
            app_name: None,
            max_pool_size: 100,
            min_pool_size: 5,
            connect_timeout_secs: 10,
            server_selection_timeout_secs: 30,
        }
    }

    /// Create a MongoConfig with a specific database name
    ///
    /// # Example
    /// ```ignore
    /// let config = MongoConfig::with_database("mongodb://localhost:27017", "mydb");
    /// ```
    pub fn with_database(url: impl Into<String>, database: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            database: database.into(),
            app_name: None,
            max_pool_size: 100,
            min_pool_size: 5,
            connect_timeout_secs: 10,
            server_selection_timeout_secs: 30,
        }
    }

    /// Create a MongoConfig with custom pool settings
    ///
    /// # Example
    /// ```ignore
    /// let config = MongoConfig::with_pool_size(
    ///     "mongodb://localhost:27017",
    ///     "mydb",
    ///     50,  // max connections
    ///     10   // min connections
    /// );
    /// ```
    pub fn with_pool_size(
        url: impl Into<String>,
        database: impl Into<String>,
        max_pool_size: u32,
        min_pool_size: u32,
    ) -> Self {
        Self {
            url: url.into(),
            database: database.into(),
            app_name: None,
            max_pool_size,
            min_pool_size,
            connect_timeout_secs: 10,
            server_selection_timeout_secs: 30,
        }
    }

    /// Set the application name for server logs
    pub fn with_app_name(mut self, app_name: impl Into<String>) -> Self {
        self.app_name = Some(app_name.into());
        self
    }

    /// Get a reference to the MongoDB URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get the database name
    pub fn database(&self) -> &str {
        &self.database
    }
}

impl Default for MongoConfig {
    fn default() -> Self {
        Self {
            url: "mongodb://localhost:27017".to_string(),
            database: "default".to_string(),
            app_name: None,
            max_pool_size: 100,
            min_pool_size: 5,
            connect_timeout_secs: 10,
            server_selection_timeout_secs: 30,
        }
    }
}

/// Load MongoConfig from environment variables
///
/// Environment variables:
/// - `MONGODB_URL` or `MONGO_URL` (required) - MongoDB connection string
/// - `MONGODB_DATABASE` or `MONGO_DATABASE` (required) - Database name
/// - `MONGODB_APP_NAME` (optional) - Application name for server logs
/// - `MONGODB_MAX_POOL_SIZE` (optional, default: 100) - Max pool connections
/// - `MONGODB_MIN_POOL_SIZE` (optional, default: 5) - Min pool connections
/// - `MONGODB_CONNECT_TIMEOUT_SECS` (optional, default: 10)
/// - `MONGODB_SERVER_SELECTION_TIMEOUT_SECS` (optional, default: 30)
///
/// # Example
/// ```ignore
/// use database::mongodb::MongoConfig;
/// use core_config::FromEnv;
///
/// let config = MongoConfig::from_env()?;
/// ```
#[cfg(feature = "config")]
impl FromEnv for MongoConfig {
    fn from_env() -> Result<Self, ConfigError> {
        // Try MONGODB_URL first, fall back to MONGO_URL
        let url = std::env::var("MONGODB_URL")
            .or_else(|_| std::env::var("MONGO_URL"))
            .map_err(|_| ConfigError::MissingEnvVar("MONGODB_URL or MONGO_URL".to_string()))?;

        // Try MONGODB_DATABASE first, fall back to MONGO_DATABASE
        let database = std::env::var("MONGODB_DATABASE")
            .or_else(|_| std::env::var("MONGO_DATABASE"))
            .map_err(|_| {
                ConfigError::MissingEnvVar("MONGODB_DATABASE or MONGO_DATABASE".to_string())
            })?;

        let app_name = std::env::var("MONGODB_APP_NAME").ok();

        let max_pool_size = std::env::var("MONGODB_MAX_POOL_SIZE")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .map_err(|e| ConfigError::ParseError {
                key: "MONGODB_MAX_POOL_SIZE".to_string(),
                details: format!("{}", e),
            })?;

        let min_pool_size = std::env::var("MONGODB_MIN_POOL_SIZE")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .map_err(|e| ConfigError::ParseError {
                key: "MONGODB_MIN_POOL_SIZE".to_string(),
                details: format!("{}", e),
            })?;

        let connect_timeout_secs = std::env::var("MONGODB_CONNECT_TIMEOUT_SECS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .map_err(|e| ConfigError::ParseError {
                key: "MONGODB_CONNECT_TIMEOUT_SECS".to_string(),
                details: format!("{}", e),
            })?;

        let server_selection_timeout_secs = std::env::var("MONGODB_SERVER_SELECTION_TIMEOUT_SECS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .map_err(|e| ConfigError::ParseError {
                key: "MONGODB_SERVER_SELECTION_TIMEOUT_SECS".to_string(),
                details: format!("{}", e),
            })?;

        Ok(Self {
            url,
            database,
            app_name,
            max_pool_size,
            min_pool_size,
            connect_timeout_secs,
            server_selection_timeout_secs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mongo_config_new() {
        let config = MongoConfig::new("mongodb://localhost:27017");
        assert_eq!(config.url, "mongodb://localhost:27017");
        assert_eq!(config.database, "default");
        assert_eq!(config.max_pool_size, 100);
        assert_eq!(config.min_pool_size, 5);
    }

    #[test]
    fn test_mongo_config_with_database() {
        let config = MongoConfig::with_database("mongodb://localhost:27017", "mydb");
        assert_eq!(config.url, "mongodb://localhost:27017");
        assert_eq!(config.database, "mydb");
    }

    #[test]
    fn test_mongo_config_with_pool_size() {
        let config = MongoConfig::with_pool_size("mongodb://localhost:27017", "mydb", 50, 10);
        assert_eq!(config.max_pool_size, 50);
        assert_eq!(config.min_pool_size, 10);
    }

    #[test]
    fn test_mongo_config_with_app_name() {
        let config = MongoConfig::new("mongodb://localhost:27017").with_app_name("my-app");
        assert_eq!(config.app_name, Some("my-app".to_string()));
    }

    #[test]
    fn test_mongo_config_default() {
        let config = MongoConfig::default();
        assert_eq!(config.url, "mongodb://localhost:27017");
        assert_eq!(config.database, "default");
    }

    #[cfg(feature = "config")]
    #[test]
    fn test_mongo_config_from_env() {
        temp_env::with_vars(
            [
                ("MONGODB_URL", Some("mongodb://localhost:27017")),
                ("MONGODB_DATABASE", Some("testdb")),
            ],
            || {
                let config = MongoConfig::from_env();
                assert!(config.is_ok());
                let config = config.unwrap();
                assert_eq!(config.url, "mongodb://localhost:27017");
                assert_eq!(config.database, "testdb");
            },
        );
    }

    #[cfg(feature = "config")]
    #[test]
    fn test_mongo_config_from_env_fallback() {
        temp_env::with_vars(
            [
                ("MONGODB_URL", None::<&str>),
                ("MONGO_URL", Some("mongodb://fallback:27017")),
                ("MONGODB_DATABASE", None::<&str>),
                ("MONGO_DATABASE", Some("fallbackdb")),
            ],
            || {
                let config = MongoConfig::from_env();
                assert!(config.is_ok());
                let config = config.unwrap();
                assert_eq!(config.url, "mongodb://fallback:27017");
                assert_eq!(config.database, "fallbackdb");
            },
        );
    }

    #[cfg(feature = "config")]
    #[test]
    fn test_mongo_config_from_env_missing_url() {
        temp_env::with_vars(
            [
                ("MONGODB_URL", None::<&str>),
                ("MONGO_URL", None::<&str>),
                ("MONGODB_DATABASE", Some("testdb")),
            ],
            || {
                let config = MongoConfig::from_env();
                assert!(config.is_err());
            },
        );
    }
}
