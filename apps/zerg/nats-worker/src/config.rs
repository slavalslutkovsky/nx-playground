//! Configuration for the NATS worker service

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    /// NATS server URL
    pub nats_url: String,

    /// Dapr settings (for future use)
    pub dapr_http_port: u16,
    pub dapr_grpc_port: u16,

    /// Pub/sub component name (Dapr uses named components)
    pub pubsub_name: String,

    /// Worker settings
    pub worker_id: String,
    pub max_concurrent_handlers: usize,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            nats_url: env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string()),

            // Dapr sidecar ports (standard Dapr defaults)
            dapr_http_port: env::var("DAPR_HTTP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3500),
            dapr_grpc_port: env::var("DAPR_GRPC_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(50001),

            // Pub/sub component name for Dapr
            pubsub_name: env::var("PUBSUB_NAME").unwrap_or_else(|_| "nats-pubsub".to_string()),

            // Worker identification
            worker_id: env::var("WORKER_ID").unwrap_or_else(|_| {
                format!(
                    "worker-{}",
                    uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
                )
            }),

            max_concurrent_handlers: env::var("MAX_CONCURRENT_HANDLERS")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(10),
        }
    }

    /// Check if running with Dapr sidecar
    pub fn is_dapr_enabled(&self) -> bool {
        env::var("DAPR_HTTP_PORT").is_ok() || env::var("DAPR_GRPC_PORT").is_ok()
    }
}
