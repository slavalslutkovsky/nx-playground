//! Server configuration loaded from environment variables.

use std::net::SocketAddr;

/// Configuration for gRPC server.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Host to bind to (default: [::1] for IPv6 localhost)
    pub host: String,
    /// Port to listen on (default: 50051)
    pub port: u16,
    /// Enable Zstd compression (default: true)
    pub enable_compression: bool,
    /// Maximum message size for decoding (default: 8MB)
    pub max_decoding_message_size: usize,
    /// Maximum message size for encoding (default: 8MB)
    pub max_encoding_message_size: usize,
    /// TCP keepalive interval in seconds (default: 60)
    pub keepalive_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "[::1]".to_string(),
            port: 50051,
            enable_compression: true,
            max_decoding_message_size: 8 * 1024 * 1024, // 8MB
            max_encoding_message_size: 8 * 1024 * 1024, // 8MB
            keepalive_secs: 60,
        }
    }
}

impl ServerConfig {
    /// Create a new server config with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from environment variables.
    ///
    /// Reads:
    /// - `GRPC_HOST` (default: [::1])
    /// - `GRPC_PORT` (default: 50051)
    /// - `GRPC_COMPRESSION` (default: true)
    /// - `GRPC_MAX_MESSAGE_SIZE` (default: 8388608 / 8MB)
    pub fn from_env() -> Result<Self, std::env::VarError> {
        let host = std::env::var("GRPC_HOST").unwrap_or_else(|_| "[::1]".to_string());
        let port = std::env::var("GRPC_PORT")
            .unwrap_or_else(|_| "50051".to_string())
            .parse()
            .unwrap_or(50051);
        let enable_compression = std::env::var("GRPC_COMPRESSION")
            .map(|v| v != "false" && v != "0")
            .unwrap_or(true);
        let max_message_size = std::env::var("GRPC_MAX_MESSAGE_SIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8 * 1024 * 1024);

        Ok(Self {
            host,
            port,
            enable_compression,
            max_decoding_message_size: max_message_size,
            max_encoding_message_size: max_message_size,
            keepalive_secs: 60,
        })
    }

    /// Set the host to bind to.
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    /// Set the port to listen on.
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Enable or disable compression.
    pub fn with_compression(mut self, enable: bool) -> Self {
        self.enable_compression = enable;
        self
    }

    /// Set maximum message size.
    pub fn with_max_message_size(mut self, size: usize) -> Self {
        self.max_decoding_message_size = size;
        self.max_encoding_message_size = size;
        self
    }

    /// Get the socket address to bind to.
    pub fn socket_addr(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        format!("{}:{}", self.host, self.port).parse()
    }

    /// Get the address string (for logging).
    pub fn addr_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "[::1]");
        assert_eq!(config.port, 50051);
        assert!(config.enable_compression);
    }

    #[test]
    fn test_builder_pattern() {
        let config = ServerConfig::new()
            .with_host("0.0.0.0")
            .with_port(8080)
            .with_compression(false);

        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert!(!config.enable_compression);
    }
}
