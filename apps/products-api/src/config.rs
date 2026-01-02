//! Configuration for Products API

use core_config::{app_info, server::ServerConfig, AppInfo, FromEnv};
use database::mongodb::MongoConfig;
use database::redis::RedisConfig;

pub use core_config::Environment;

/// Application configuration
#[derive(Clone, Debug)]
pub struct Config {
    pub app: AppInfo,
    pub mongodb: MongoConfig,
    pub redis: RedisConfig,
    pub server: ServerConfig,
    pub environment: Environment,
    pub grpc_port: u16,
    pub dapr_http_port: Option<u16>,
}

impl Config {
    pub fn from_env() -> eyre::Result<Self> {
        let environment = Environment::from_env();
        let mongodb = MongoConfig::from_env()?;
        let redis = RedisConfig::from_env()?;
        let server = ServerConfig::from_env()?;

        let grpc_port = std::env::var("GRPC_PORT")
            .unwrap_or_else(|_| "50051".to_string())
            .parse()
            .unwrap_or(50051);

        let dapr_http_port = std::env::var("DAPR_HTTP_PORT")
            .ok()
            .and_then(|p| p.parse().ok());

        Ok(Self {
            app: app_info!(),
            mongodb,
            redis,
            server,
            environment,
            grpc_port,
            dapr_http_port,
        })
    }
}
