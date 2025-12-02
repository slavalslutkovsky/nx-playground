
use core_config::{app_info, server::ServerConfig, AppInfo, FromEnv};

// Import database configs from the database library
use database::postgres::PostgresConfig;
use database::redis::RedisConfig;

// Re-export Environment for use in other modules
pub use core_config::Environment;

/// Application-specific configuration
/// Composes shared config components from the `config` library
#[derive(Clone, Debug)]
pub struct Config {
    pub app: AppInfo,
    pub database: PostgresConfig,
    pub redis: RedisConfig,
    pub server: ServerConfig,
    pub environment: Environment,
}

impl Config {
    pub fn from_env() -> eyre::Result<Self> {
        let environment = Environment::from_env();
        let database = PostgresConfig::from_env()?; // Required - will fail if not set
        let server = ServerConfig::from_env()?; // Uses defaults: HOST=0.0.0.0, PORT=8080
        let redis = RedisConfig::from_env()?; // Required - will fail if not set

        Ok(Self {
            app: app_info!(),
            database,
            redis,
            server,
            environment,
        })
    }
}
