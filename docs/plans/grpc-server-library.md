# Plan: gRPC Server Shared Library

Extract common gRPC server boilerplate from `zerg-tasks` and `zerg-vector` into `libs/core/grpc`.

## Problem

The `server.rs` files in both services have ~40 lines of identical boilerplate:

```rust
// Repeated in both tasks/server.rs and vector/server.rs
let host = std::env::var("GRPC_HOST").unwrap_or_else(|_| "[::1]".to_string());
let port = std::env::var("GRPC_PORT").unwrap_or_else(|_| "50051".to_string());
let addr_str = format!("{}:{}", host, port);
let addr = addr_str.parse()?;

let (health_reporter, health_service) = health_reporter();
health_reporter.set_service_status(SERVICE_NAME, ServingStatus::Serving).await;
health_reporter.set_service_status("", ServingStatus::Serving).await;

Server::builder()
    .add_service(health_service)
    .add_service(MyServiceServer::new(service)
        .accept_compressed(CompressionEncoding::Zstd)
        .send_compressed(CompressionEncoding::Zstd))
    .serve(addr)
    .await?;
```

## Analysis

| Component | tasks/server.rs | vector/server.rs | Shareable? |
|-----------|-----------------|------------------|------------|
| Tracing init | `init_tracing(&env)` | Same | Already in `core_config` |
| Address parsing | `GRPC_HOST`, `GRPC_PORT` | Same | **Yes** |
| Health reporter | `health_reporter()` | Same | **Yes** |
| Service status | `set_service_status()` | Same | **Yes** |
| Server builder | `Server::builder()` | Same | **Yes** |
| Compression | Zstd | Zstd | **Yes** |
| DB connection | Postgres | Qdrant | No (domain-specific) |
| Service creation | TaskService | VectorService | No (domain-specific) |

## Solution

Extend `libs/core/grpc` with a `server` module. This library already handles client-side utilities; adding server-side completes it.

### Current Structure

```
libs/core/grpc/src/
├── lib.rs           # Client utilities
├── channel.rs       # Channel creation
├── client.rs        # Client configuration
├── retry.rs         # Retry logic
├── error.rs         # Error types
├── conversions.rs   # Type conversions
└── interceptors.rs  # Auth/tracing interceptors
```

### Proposed Structure

```
libs/core/grpc/src/
├── lib.rs
├── channel.rs
├── client.rs
├── retry.rs
├── error.rs
├── conversions.rs
├── interceptors.rs
└── server/              # NEW
    ├── mod.rs           # Module exports
    ├── config.rs        # GrpcServerConfig
    ├── health.rs        # Health check helpers
    └── builder.rs       # GrpcServerBuilder
```

## API Design

### GrpcServerConfig

```rust
// libs/core/grpc/src/server/config.rs

/// Configuration for gRPC server address
#[derive(Clone, Debug)]
pub struct GrpcServerConfig {
    pub host: String,
    pub port: u16,
}

impl GrpcServerConfig {
    /// Load from environment with default port
    pub fn from_env_with_default(default_port: u16) -> Result<Self, ConfigError> {
        let host = std::env::var("GRPC_HOST")
            .unwrap_or_else(|_| "[::1]".to_string());
        let port = std::env::var("GRPC_PORT")
            .map(|p| p.parse().unwrap_or(default_port))
            .unwrap_or(default_port);

        Ok(Self { host, port })
    }

    /// Get socket address
    pub fn socket_addr(&self) -> Result<SocketAddr, AddrParseError> {
        format!("{}:{}", self.host, self.port).parse()
    }
}
```

### GrpcServerBuilder

```rust
// libs/core/grpc/src/server/builder.rs

use tonic::codec::CompressionEncoding;
use tonic::transport::Server;
use tonic_health::server::health_reporter;

/// Builder for gRPC servers with standard configuration
pub struct GrpcServerBuilder {
    config: GrpcServerConfig,
    service_name: &'static str,
    compression: CompressionEncoding,
    enable_health: bool,
}

impl GrpcServerBuilder {
    /// Create new builder with service name and default port
    pub fn new(service_name: &'static str, default_port: u16) -> Result<Self> {
        Ok(Self {
            config: GrpcServerConfig::from_env_with_default(default_port)?,
            service_name,
            compression: CompressionEncoding::Zstd,
            enable_health: true,
        })
    }

    /// Use custom config instead of env vars
    pub fn with_config(mut self, config: GrpcServerConfig) -> Self {
        self.config = config;
        self
    }

    /// Set compression encoding (default: Zstd)
    pub fn with_compression(mut self, encoding: CompressionEncoding) -> Self {
        self.compression = encoding;
        self
    }

    /// Disable health check service
    pub fn without_health_check(mut self) -> Self {
        self.enable_health = false;
        self
    }

    /// Serve a gRPC service
    pub async fn serve<S>(self, service: S) -> Result<()>
    where
        S: tonic::codegen::Service<
            http::Request<tonic::body::BoxBody>,
            Response = http::Response<tonic::body::BoxBody>,
        > + Clone + Send + 'static,
        S::Future: Send,
    {
        let addr = self.config.socket_addr()?;

        info!("{} listening on {}", self.service_name, addr);
        info!("Compression: {:?}", self.compression);

        let mut builder = Server::builder();

        if self.enable_health {
            let (health_reporter, health_service) = health_reporter();

            // Mark service as serving for K8s probes
            health_reporter
                .set_service_status(self.service_name, tonic_health::ServingStatus::Serving)
                .await;
            health_reporter
                .set_service_status("", tonic_health::ServingStatus::Serving)
                .await;

            info!("Health check enabled (grpc.health.v1.Health)");
            builder = builder.add_service(health_service);
        }

        builder
            .add_service(service)
            .serve(addr)
            .await?;

        Ok(())
    }
}
```

### Helper Function for Compression

```rust
// libs/core/grpc/src/server/mod.rs

/// Apply standard compression to a gRPC service
pub fn with_server_compression<S>(service: S) -> S
where
    S: tonic::server::NamedService,
{
    service
        .accept_compressed(CompressionEncoding::Zstd)
        .send_compressed(CompressionEncoding::Zstd)
}
```

## Usage Examples

### Before (tasks/server.rs - 97 lines)

```rust
pub async fn run() -> Result<()> {
    let environment = Environment::from_env();
    core_config::tracing::init_tracing(&environment);

    let config = PostgresConfig::from_env()?;
    let db = connect_from_config_with_retry(config, None).await?;

    let repository = PgTaskRepository::new(db);
    let service = TaskService::new(repository);
    let tasks_service = TasksServiceImpl::new(service);

    let host = std::env::var("GRPC_HOST").unwrap_or_else(|_| "[::1]".to_string());
    let port = std::env::var("GRPC_PORT").unwrap_or_else(|_| "50051".to_string());
    let addr_str = format!("{}:{}", host, port);
    let addr = addr_str.parse()?;

    info!("TasksService listening on {}", addr);
    info!("Using Zstd compression");

    let (health_reporter, health_service) = health_reporter();
    health_reporter.set_service_status(SERVICE_NAME, ServingStatus::Serving).await;
    health_reporter.set_service_status("", ServingStatus::Serving).await;
    info!("Health check service enabled");

    Server::builder()
        .add_service(health_service)
        .add_service(
            TasksServiceServer::new(tasks_service)
                .accept_compressed(CompressionEncoding::Zstd)
                .send_compressed(CompressionEncoding::Zstd),
        )
        .serve(addr)
        .await?;

    Ok(())
}
```

### After (tasks/server.rs - ~25 lines)

```rust
use grpc::server::{GrpcServerBuilder, with_server_compression};

pub async fn run() -> Result<()> {
    // Tracing (already shared)
    core_config::tracing::init_from_env();

    // Domain-specific setup (cannot be shared)
    let config = PostgresConfig::from_env()?;
    let db = connect_from_config_with_retry(config, None).await?;
    let repository = PgTaskRepository::new(db);
    let service = TaskService::new(repository);
    let tasks_service = TasksServiceImpl::new(service);

    // Server setup (now 3 lines instead of 25+)
    GrpcServerBuilder::new(SERVICE_NAME, 50051)?
        .serve(with_server_compression(TasksServiceServer::new(tasks_service)))
        .await
}
```

### After (vector/server.rs - ~30 lines)

```rust
use grpc::server::{GrpcServerBuilder, with_server_compression};

pub async fn run() -> Result<()> {
    core_config::tracing::init_from_env();

    // Domain-specific: Qdrant + optional embedding provider
    let qdrant_config = QdrantConfig::from_env()?;
    let repository = QdrantRepository::new(qdrant_config).await?;
    let mut service = VectorService::new(repository);

    if let Ok(provider) = OpenAIProvider::from_env() {
        service = service.with_embedding_provider(Arc::new(provider));
    }

    let vector_service = VectorServiceImpl::new(service);

    // Server setup (shared)
    GrpcServerBuilder::new(SERVICE_NAME, 50052)?
        .serve(with_server_compression(VectorServiceServer::new(vector_service)))
        .await
}
```

## Implementation Steps

| Step | Task | Files | Effort |
|------|------|-------|--------|
| 1 | Add `tonic-health` dependency | `libs/core/grpc/Cargo.toml` | 5 min |
| 2 | Create `server/mod.rs` | `libs/core/grpc/src/server/mod.rs` | 5 min |
| 3 | Create `GrpcServerConfig` | `libs/core/grpc/src/server/config.rs` | 15 min |
| 4 | Create `GrpcServerBuilder` | `libs/core/grpc/src/server/builder.rs` | 30 min |
| 5 | Add compression helper | `libs/core/grpc/src/server/mod.rs` | 10 min |
| 6 | Export from lib.rs | `libs/core/grpc/src/lib.rs` | 5 min |
| 7 | Add tests | `libs/core/grpc/src/server/tests.rs` | 30 min |
| 8 | Refactor tasks/server.rs | `apps/zerg/tasks/src/server.rs` | 15 min |
| 9 | Refactor vector/server.rs | `apps/zerg/vector/src/server.rs` | 15 min |
| 10 | Update documentation | `libs/core/grpc/README.md` | 10 min |

**Total estimated effort: ~2.5 hours**

## Dependencies to Add

```toml
# libs/core/grpc/Cargo.toml
[dependencies]
tonic-health = "0.12"  # Health check service
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env_defaults() {
        temp_env::with_vars_unset(["GRPC_HOST", "GRPC_PORT"], || {
            let config = GrpcServerConfig::from_env_with_default(50051).unwrap();
            assert_eq!(config.host, "[::1]");
            assert_eq!(config.port, 50051);
        });
    }

    #[test]
    fn test_config_from_env_custom() {
        temp_env::with_vars([
            ("GRPC_HOST", Some("0.0.0.0")),
            ("GRPC_PORT", Some("9000")),
        ], || {
            let config = GrpcServerConfig::from_env_with_default(50051).unwrap();
            assert_eq!(config.host, "0.0.0.0");
            assert_eq!(config.port, 9000);
        });
    }

    #[test]
    fn test_socket_addr_parsing() {
        let config = GrpcServerConfig {
            host: "[::1]".to_string(),
            port: 50051,
        };
        assert!(config.socket_addr().is_ok());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_grpc_server_starts() {
    // Create a mock service
    let service = MockService::new();

    // Start server in background
    let handle = tokio::spawn(async {
        GrpcServerBuilder::new("test.MockService", 0)  // port 0 = random
            .serve(MockServiceServer::new(service))
            .await
    });

    // Verify health check responds
    // ...

    handle.abort();
}
```

## Future Enhancements

1. **Graceful Shutdown**: Add signal handling for SIGTERM/SIGINT
2. **Reflection**: Optional gRPC reflection for debugging
3. **Metrics**: Prometheus metrics integration
4. **TLS**: Optional TLS configuration
5. **Interceptors**: Server-side interceptor support

## Alternatives Considered

### 1. Macro Approach

```rust
grpc_serve!(TasksServiceServer::new(service), SERVICE_NAME, 50051);
```

**Pros**: Even more concise
**Cons**: Less flexible, harder to debug, magic

### 2. Trait-Based Approach

```rust
impl GrpcService for TasksServer {
    fn service_name() -> &'static str { SERVICE_NAME }
    fn default_port() -> u16 { 50051 }
    async fn create_service(&self) -> impl Service { ... }
}
```

**Pros**: More structured
**Cons**: More boilerplate, over-engineered for 2 services

### 3. Separate Library

Create `libs/core/grpc-server` instead of extending `libs/core/grpc`.

**Pros**: Clear separation
**Cons**: Another dependency to manage, grpc lib is the natural home

## Decision

Use **Builder Pattern** in `libs/core/grpc/src/server/`:
- Flexible and composable
- Familiar Rust pattern
- Easy to extend
- Clear, readable code
- Natural home alongside client utilities

## Related Files

- `apps/zerg/tasks/src/server.rs` - Current tasks server
- `apps/zerg/vector/src/server.rs` - Current vector server
- `libs/core/grpc/src/lib.rs` - Existing gRPC client utilities
- `libs/core/config/src/server.rs` - HTTP server config (similar pattern)
