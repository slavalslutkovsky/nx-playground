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
    // Auth configuration
    pub session_secret: String,
    pub cors_allowed_origin: String,
    pub frontend_url: String,
    pub redirect_base_url: String,
    // OAuth configuration
    pub google_client_id: String,
    pub google_client_secret: String,
    pub github_client_id: String,
    pub github_client_secret: String,
}

impl Config {
    pub fn from_env() -> eyre::Result<Self> {
        let environment = Environment::from_env();
        let database = PostgresConfig::from_env()?; // Required - will fail if not set
        let server = ServerConfig::from_env()?; // Uses defaults: HOST=0.0.0.0, PORT=8080
        let redis = RedisConfig::from_env()?; // Required - will fail if not set

        // Auth configuration
        let session_secret = core_config::env_required("JWT_SECRET")?;
        let cors_allowed_origin = core_config::env_or_default("CORS_ALLOWED_ORIGIN", "http://localhost:3000");
        let frontend_url = core_config::env_or_default("FRONTEND_URL", "http://localhost:3000");
        let redirect_base_url = core_config::env_or_default("REDIRECT_BASE_URL", "http://localhost:8080");

        // OAuth configuration
        let google_client_id = core_config::env_required("GOOGLE_CLIENT_ID")?;
        let google_client_secret = core_config::env_required("GOOGLE_CLIENT_SECRET")?;
        let github_client_id = core_config::env_required("GITHUB_CLIENT_ID")?;
        let github_client_secret = core_config::env_required("GITHUB_CLIENT_SECRET")?;

        Ok(Self {
            app: app_info!(),
            database,
            redis,
            server,
            environment,
            session_secret,
            cors_allowed_origin,
            frontend_url,
            redirect_base_url,
            google_client_id,
            google_client_secret,
            github_client_id,
            github_client_secret,
        })
    }
}
