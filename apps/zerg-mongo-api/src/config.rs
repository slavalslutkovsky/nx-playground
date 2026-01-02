use core_config::{AppInfo, FromEnv, app_info, server::ServerConfig};

// Import MongoDB config from the database library
use database::mongodb::MongoConfig;

// Re-export Environment for use in other modules
pub use core_config::Environment;

/// Application-specific configuration
/// Composes shared config components from the `config` library
#[derive(Clone, Debug)]
pub struct Config {
    pub app: AppInfo,
    pub mongodb: MongoConfig,
    pub server: ServerConfig,
    pub environment: Environment,
}

impl Config {
    pub fn from_env() -> eyre::Result<Self> {
        let environment = Environment::from_env();
        let mongodb = MongoConfig::from_env()?;
        let server = ServerConfig::from_env()?;

        Ok(Self {
            app: app_info!(),
            mongodb,
            server,
            environment,
        })
    }
}
