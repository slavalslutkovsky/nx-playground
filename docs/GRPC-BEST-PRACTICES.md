# gRPC Best Practices Guide

A comprehensive guide covering gRPC design patterns, security, performance optimization, and resources.

---

## Table of Contents

1. [API Design](#api-design)
2. [Error Handling](#error-handling)
3. [Performance Optimization](#performance-optimization)
4. [Streaming Patterns](#streaming-patterns)
5. [Security](#security)
6. [Testing](#testing)
7. [Observability](#observability)
8. [Framework Examples](#framework-examples)
   - [NestJS gRPC](#nestjs-grpc-implementation)
   - [Hono + ConnectRPC](#hono-with-connectrpc)
9. [Common Pitfalls](#common-pitfalls)
10. [Resources](#resources)

---

## API Design

### Proto File Organization

```
proto/
├── buf.yaml                 # buf configuration
├── buf.gen.yaml             # code generation config
├── common/
│   └── v1/
│       ├── pagination.proto
│       └── errors.proto
├── user/
│   └── v1/
│       └── user.proto
└── order/
    └── v1/
        └── order.proto
```

### Versioning Strategy

**Always version your APIs from day one:**

```protobuf
syntax = "proto3";

package mycompany.user.v1;  // Version in package name

option go_package = "github.com/mycompany/api/user/v1;userv1";
option java_package = "com.mycompany.api.user.v1";
```

### Naming Conventions

```protobuf
// Service: PascalCase + "Service" suffix
service UserService {
  // RPC: PascalCase, verb-noun pattern
  rpc GetUser(GetUserRequest) returns (GetUserResponse);
  rpc ListUsers(ListUsersRequest) returns (ListUsersResponse);
  rpc CreateUser(CreateUserRequest) returns (CreateUserResponse);
  rpc UpdateUser(UpdateUserRequest) returns (UpdateUserResponse);
  rpc DeleteUser(DeleteUserRequest) returns (DeleteUserResponse);
}

// Messages: PascalCase
// Fields: snake_case
message User {
  string user_id = 1;           // NOT: userId, id
  string email_address = 2;     // NOT: emailAddress
  int64 created_at_unix = 3;    // Be explicit about units
}

// Enums: SCREAMING_SNAKE_CASE with prefix
enum UserStatus {
  USER_STATUS_UNSPECIFIED = 0;  // Always have UNSPECIFIED as 0
  USER_STATUS_ACTIVE = 1;
  USER_STATUS_INACTIVE = 2;
  USER_STATUS_BANNED = 3;
}
```

### Request/Response Design

```protobuf
// Always use dedicated request/response messages (not primitives)
// BAD:
rpc GetUser(string) returns (User);

// GOOD:
rpc GetUser(GetUserRequest) returns (GetUserResponse);

message GetUserRequest {
  string user_id = 1;

  // Use field masks for partial responses
  google.protobuf.FieldMask read_mask = 2;
}

message GetUserResponse {
  User user = 1;
}
```

### Pagination

```protobuf
import "google/protobuf/field_mask.proto";

message ListUsersRequest {
  // Cursor-based pagination (preferred)
  string page_token = 1;
  int32 page_size = 2;  // Max items per page

  // Optional filtering
  string filter = 3;  // e.g., "status=ACTIVE AND created_at > 2024-01-01"

  // Optional ordering
  string order_by = 4;  // e.g., "created_at desc, name asc"

  // Field mask for partial responses
  google.protobuf.FieldMask read_mask = 5;
}

message ListUsersResponse {
  repeated User users = 1;
  string next_page_token = 2;  // Empty if no more pages
  int32 total_size = 3;        // Optional: total count (can be expensive)
}
```

### Resource-Oriented Design

Follow Google's AIP (API Improvement Proposals):

```protobuf
// Resources have standard methods:
// - Get: Retrieve single resource
// - List: Retrieve collection
// - Create: Create new resource
// - Update: Modify existing resource
// - Delete: Remove resource

service BookService {
  // Standard methods
  rpc GetBook(GetBookRequest) returns (Book);
  rpc ListBooks(ListBooksRequest) returns (ListBooksResponse);
  rpc CreateBook(CreateBookRequest) returns (Book);
  rpc UpdateBook(UpdateBookRequest) returns (Book);
  rpc DeleteBook(DeleteBookRequest) returns (google.protobuf.Empty);

  // Custom methods use colon syntax in HTTP transcoding
  rpc ArchiveBook(ArchiveBookRequest) returns (Book);
}
```

---

## Error Handling

### Use Standard gRPC Status Codes

| Code | Name | Use Case |
|------|------|----------|
| 0 | OK | Success |
| 1 | CANCELLED | Client cancelled request |
| 2 | UNKNOWN | Unknown error (avoid) |
| 3 | INVALID_ARGUMENT | Bad request parameters |
| 4 | DEADLINE_EXCEEDED | Timeout |
| 5 | NOT_FOUND | Resource doesn't exist |
| 6 | ALREADY_EXISTS | Resource already exists |
| 7 | PERMISSION_DENIED | Authenticated but not authorized |
| 8 | RESOURCE_EXHAUSTED | Rate limited / quota exceeded |
| 9 | FAILED_PRECONDITION | System not in required state |
| 10 | ABORTED | Concurrency conflict |
| 11 | OUT_OF_RANGE | Valid type but invalid range |
| 12 | UNIMPLEMENTED | Method not implemented |
| 13 | INTERNAL | Internal server error |
| 14 | UNAVAILABLE | Service temporarily unavailable |
| 16 | UNAUTHENTICATED | No valid credentials |

### Rich Error Details

```protobuf
import "google/rpc/status.proto";
import "google/rpc/error_details.proto";

// Use google.rpc.Status with details for rich errors
```

**Rust Example (tonic):**

```rust
use tonic::{Code, Status};
use prost_types::Any;

// Simple error
fn simple_error() -> Status {
    Status::not_found("User not found")
}

// Rich error with details
fn rich_error(user_id: &str) -> Status {
    let mut status = Status::invalid_argument("Validation failed");

    // Add field violation details
    let details = BadRequest {
        field_violations: vec![
            FieldViolation {
                field: "email".to_string(),
                description: "Invalid email format".to_string(),
            },
            FieldViolation {
                field: "age".to_string(),
                description: "Must be >= 18".to_string(),
            },
        ],
    };

    status
}
```

**Go Example:**

```go
import (
    "google.golang.org/grpc/codes"
    "google.golang.org/grpc/status"
    "google.golang.org/genproto/googleapis/rpc/errdetails"
)

func richError() error {
    st := status.New(codes.InvalidArgument, "Validation failed")

    // Add field violations
    br := &errdetails.BadRequest{
        FieldViolations: []*errdetails.BadRequest_FieldViolation{
            {Field: "email", Description: "Invalid email format"},
            {Field: "age", Description: "Must be >= 18"},
        },
    }

    st, _ = st.WithDetails(br)
    return st.Err()
}
```

### Error Handling Best Practices

1. **Don't expose internal details** - Sanitize errors before returning to clients
2. **Log server-side, return client-safe messages** - Log stack traces internally
3. **Use UNAVAILABLE for retryable errors** - Clients will retry
4. **Use INTERNAL sparingly** - Usually indicates a bug
5. **Include request IDs** - For correlation in distributed tracing

```rust
// Rust: Sanitize errors
impl From<sqlx::Error> for Status {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {:?}", err);

        match err {
            sqlx::Error::RowNotFound => Status::not_found("Resource not found"),
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                Status::already_exists("Resource already exists")
            }
            _ => Status::internal("Internal error"),  // Don't leak DB details
        }
    }
}
```

---

## Performance Optimization

### Connection Management

```rust
// Rust (tonic): Reuse channels, don't create per-request
use tonic::transport::Channel;

// GOOD: Create channel once, reuse
lazy_static! {
    static ref CHANNEL: Channel = Channel::from_static("http://[::1]:50051")
        .connect_lazy();
}

// BAD: Creating new connection per request
async fn bad_call() {
    let channel = Channel::from_static("http://[::1]:50051")
        .connect()
        .await
        .unwrap();  // Connection overhead on every call!
}
```

### Keep-Alive Configuration

```rust
// Rust (tonic)
use std::time::Duration;
use tonic::transport::Channel;

let channel = Channel::from_static("http://[::1]:50051")
    .keep_alive_timeout(Duration::from_secs(20))
    .keep_alive_while_idle(true)
    .http2_keep_alive_interval(Duration::from_secs(10))
    .connect_lazy();
```

```go
// Go
import "google.golang.org/grpc/keepalive"

conn, err := grpc.Dial(address,
    grpc.WithKeepaliveParams(keepalive.ClientParameters{
        Time:                10 * time.Second,
        Timeout:             20 * time.Second,
        PermitWithoutStream: true,
    }),
)
```

### Message Size Limits

```rust
// Rust (tonic) - Server
Server::builder()
    .max_decoding_message_size(16 * 1024 * 1024)  // 16MB
    .max_encoding_message_size(16 * 1024 * 1024)
    .add_service(svc)
    .serve(addr)
    .await?;

// Client
let channel = Channel::from_static("http://[::1]:50051")
    .connect()
    .await?;

let client = UserServiceClient::new(channel)
    .max_decoding_message_size(16 * 1024 * 1024)
    .max_encoding_message_size(16 * 1024 * 1024);
```

### Compression

```rust
// Rust (tonic)
use tonic::codec::CompressionEncoding;

// Server
Server::builder()
    .add_service(
        UserServiceServer::new(my_service)
            .send_compressed(CompressionEncoding::Gzip)
            .accept_compressed(CompressionEncoding::Gzip)
    )
    .serve(addr)
    .await?;

// Client
let client = UserServiceClient::new(channel)
    .send_compressed(CompressionEncoding::Gzip)
    .accept_compressed(CompressionEncoding::Gzip);
```

### Load Balancing

```protobuf
// DNS-based with multiple A records
// grpclb (deprecated) -> xDS (modern)
```

```go
// Go: Round-robin load balancing
import _ "google.golang.org/grpc/balancer/roundrobin"

conn, err := grpc.Dial(
    "dns:///my-service:50051",
    grpc.WithDefaultServiceConfig(`{"loadBalancingPolicy":"round_robin"}`),
)
```

### Deadlines (Always Set Them!)

```rust
// Rust (tonic)
use std::time::Duration;
use tonic::Request;

let mut request = Request::new(GetUserRequest { user_id: "123".into() });
request.set_timeout(Duration::from_secs(5));

let response = client.get_user(request).await?;
```

```go
// Go
ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
defer cancel()

resp, err := client.GetUser(ctx, &pb.GetUserRequest{UserId: "123"})
```

---

## Streaming Patterns

### Server Streaming

Use for: Large datasets, real-time feeds, logs

```protobuf
service LogService {
  rpc StreamLogs(StreamLogsRequest) returns (stream LogEntry);
}
```

```rust
// Rust (tonic) Server
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

#[tonic::async_trait]
impl LogService for MyLogService {
    type StreamLogsStream = ReceiverStream<Result<LogEntry, Status>>;

    async fn stream_logs(
        &self,
        request: Request<StreamLogsRequest>,
    ) -> Result<Response<Self::StreamLogsStream>, Status> {
        let (tx, rx) = mpsc::channel(128);

        tokio::spawn(async move {
            // Stream logs from source
            while let Some(log) = get_next_log().await {
                if tx.send(Ok(log)).await.is_err() {
                    break;  // Client disconnected
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
```

### Client Streaming

Use for: File uploads, batched writes

```protobuf
service UploadService {
  rpc UploadFile(stream FileChunk) returns (UploadResponse);
}
```

```rust
// Rust (tonic) Server
async fn upload_file(
    &self,
    request: Request<tonic::Streaming<FileChunk>>,
) -> Result<Response<UploadResponse>, Status> {
    let mut stream = request.into_inner();
    let mut total_bytes = 0;

    while let Some(chunk) = stream.message().await? {
        // Process chunk
        total_bytes += chunk.data.len();
    }

    Ok(Response::new(UploadResponse {
        bytes_received: total_bytes as i64,
    }))
}
```

### Bidirectional Streaming

Use for: Chat, multiplayer games, collaborative editing

```protobuf
service ChatService {
  rpc Chat(stream ChatMessage) returns (stream ChatMessage);
}
```

```rust
// Rust (tonic) Server
async fn chat(
    &self,
    request: Request<tonic::Streaming<ChatMessage>>,
) -> Result<Response<Self::ChatStream>, Status> {
    let mut inbound = request.into_inner();
    let (tx, rx) = mpsc::channel(128);

    tokio::spawn(async move {
        while let Some(Ok(msg)) = inbound.message().await {
            // Echo back (in real app, broadcast to other clients)
            let response = ChatMessage {
                user: msg.user,
                text: format!("Echo: {}", msg.text),
            };
            if tx.send(Ok(response)).await.is_err() {
                break;
            }
        }
    });

    Ok(Response::new(ReceiverStream::new(rx)))
}
```

### Streaming Best Practices

1. **Implement flow control** - Don't overwhelm slow consumers
2. **Handle cancellation** - Clean up resources when client disconnects
3. **Use keep-alives for long streams** - Prevent idle timeouts
4. **Chunk large messages** - Stay under message size limits
5. **Consider deadlines** - Streams can run indefinitely

---

## Security

### TLS (Transport Layer Security)

**Always use TLS in production!**

```rust
// Rust (tonic) Server with TLS
use tonic::transport::{Server, Identity, ServerTlsConfig};

let cert = tokio::fs::read("server.pem").await?;
let key = tokio::fs::read("server.key").await?;
let identity = Identity::from_pem(cert, key);

Server::builder()
    .tls_config(ServerTlsConfig::new().identity(identity))?
    .add_service(svc)
    .serve(addr)
    .await?;
```

```rust
// Rust (tonic) Client with TLS
use tonic::transport::{Certificate, ClientTlsConfig};

let ca_cert = tokio::fs::read("ca.pem").await?;
let tls = ClientTlsConfig::new()
    .ca_certificate(Certificate::from_pem(ca_cert))
    .domain_name("my-service.example.com");

let channel = Channel::from_static("https://my-service:50051")
    .tls_config(tls)?
    .connect()
    .await?;
```

### mTLS (Mutual TLS)

```rust
// Server with mTLS (requires client certificate)
use tonic::transport::{Certificate, Identity, ServerTlsConfig};

let cert = tokio::fs::read("server.pem").await?;
let key = tokio::fs::read("server.key").await?;
let ca_cert = tokio::fs::read("ca.pem").await?;

let tls = ServerTlsConfig::new()
    .identity(Identity::from_pem(cert, key))
    .client_ca_root(Certificate::from_pem(ca_cert));  // Require client cert

Server::builder()
    .tls_config(tls)?
    .add_service(svc)
    .serve(addr)
    .await?;
```

```rust
// Client with mTLS
let client_cert = tokio::fs::read("client.pem").await?;
let client_key = tokio::fs::read("client.key").await?;
let ca_cert = tokio::fs::read("ca.pem").await?;

let tls = ClientTlsConfig::new()
    .identity(Identity::from_pem(client_cert, client_key))
    .ca_certificate(Certificate::from_pem(ca_cert))
    .domain_name("my-service");

let channel = Channel::from_static("https://my-service:50051")
    .tls_config(tls)?
    .connect()
    .await?;
```

### Authentication Patterns

#### Token-Based (JWT/OAuth2)

```rust
// Rust: Interceptor for adding auth token
use tonic::{Request, Status};
use tonic::service::Interceptor;

#[derive(Clone)]
pub struct AuthInterceptor {
    token: String,
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        request.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", self.token).parse().unwrap(),
        );
        Ok(request)
    }
}

// Usage
let channel = Channel::from_static("https://my-service:50051")
    .connect()
    .await?;

let client = UserServiceClient::with_interceptor(
    channel,
    AuthInterceptor { token: "my-jwt-token".into() },
);
```

```rust
// Server: Validate token in interceptor
use tonic::{Request, Status};

fn check_auth<T>(req: Request<T>) -> Result<Request<T>, Status> {
    match req.metadata().get("authorization") {
        Some(token) => {
            let token = token.to_str().map_err(|_| Status::unauthenticated("Invalid token"))?;

            if token.starts_with("Bearer ") {
                let jwt = &token[7..];
                // Validate JWT here
                validate_jwt(jwt)?;
                Ok(req)
            } else {
                Err(Status::unauthenticated("Missing bearer prefix"))
            }
        }
        None => Err(Status::unauthenticated("Missing authorization")),
    }
}

// Apply to service
Server::builder()
    .add_service(UserServiceServer::with_interceptor(my_service, check_auth))
    .serve(addr)
    .await?;
```

#### API Key Authentication

```rust
fn check_api_key<T>(req: Request<T>) -> Result<Request<T>, Status> {
    match req.metadata().get("x-api-key") {
        Some(key) => {
            let key = key.to_str().map_err(|_| Status::unauthenticated("Invalid key"))?;

            if is_valid_api_key(key) {
                Ok(req)
            } else {
                Err(Status::unauthenticated("Invalid API key"))
            }
        }
        None => Err(Status::unauthenticated("Missing API key")),
    }
}
```

### Authorization (RBAC/ABAC)

```rust
use std::collections::HashSet;

struct AuthContext {
    user_id: String,
    roles: HashSet<String>,
}

impl AuthContext {
    fn has_role(&self, role: &str) -> bool {
        self.roles.contains(role)
    }

    fn can_access_resource(&self, resource_owner: &str) -> bool {
        self.user_id == resource_owner || self.has_role("admin")
    }
}

// In service implementation
async fn delete_user(
    &self,
    request: Request<DeleteUserRequest>,
) -> Result<Response<()>, Status> {
    let auth = extract_auth_context(&request)?;
    let req = request.into_inner();

    // Check authorization
    if !auth.can_access_resource(&req.user_id) {
        return Err(Status::permission_denied("Cannot delete other users"));
    }

    // Proceed with deletion
    self.repo.delete_user(&req.user_id).await?;
    Ok(Response::new(()))
}
```

### Rate Limiting

```rust
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

// Create rate limiter
let limiter = RateLimiter::direct(Quota::per_second(NonZeroU32::new(100).unwrap()));

// In interceptor
fn rate_limit<T>(req: Request<T>) -> Result<Request<T>, Status> {
    match limiter.check() {
        Ok(_) => Ok(req),
        Err(_) => Err(Status::resource_exhausted("Rate limit exceeded")),
    }
}
```

### Security Checklist

- [ ] **TLS everywhere** - No plaintext gRPC in production
- [ ] **Validate all inputs** - Don't trust client data
- [ ] **Set message size limits** - Prevent DoS via large messages
- [ ] **Implement timeouts** - Prevent resource exhaustion
- [ ] **Rate limit** - Protect against abuse
- [ ] **Authenticate** - Know who's calling
- [ ] **Authorize** - Verify they can do what they're asking
- [ ] **Audit log** - Record sensitive operations
- [ ] **Don't leak errors** - Sanitize before returning
- [ ] **Keep dependencies updated** - Watch for CVEs

---

## Testing

### Unit Testing

```rust
// Rust: Test service implementation directly
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_user() {
        let service = MyUserService::new(mock_repo());

        let request = Request::new(GetUserRequest {
            user_id: "123".into(),
        });

        let response = service.get_user(request).await.unwrap();
        assert_eq!(response.get_ref().user.unwrap().user_id, "123");
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let service = MyUserService::new(mock_repo());

        let request = Request::new(GetUserRequest {
            user_id: "nonexistent".into(),
        });

        let err = service.get_user(request).await.unwrap_err();
        assert_eq!(err.code(), Code::NotFound);
    }
}
```

### Integration Testing

```rust
// Rust: Spin up server and test with real client
#[tokio::test]
async fn integration_test() {
    // Start server in background
    let addr = "[::1]:50051".parse().unwrap();
    let service = MyUserService::new(test_db());

    tokio::spawn(async move {
        Server::builder()
            .add_service(UserServiceServer::new(service))
            .serve(addr)
            .await
            .unwrap();
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Test with real client
    let channel = Channel::from_static("http://[::1]:50051")
        .connect()
        .await
        .unwrap();

    let mut client = UserServiceClient::new(channel);

    let response = client
        .get_user(GetUserRequest { user_id: "123".into() })
        .await
        .unwrap();

    assert!(response.into_inner().user.is_some());
}
```

### Testing with grpcurl

```bash
# List services
grpcurl -plaintext localhost:50051 list

# Describe service
grpcurl -plaintext localhost:50051 describe mycompany.user.v1.UserService

# Call method
grpcurl -plaintext -d '{"user_id": "123"}' \
  localhost:50051 mycompany.user.v1.UserService/GetUser

# With TLS
grpcurl -cacert ca.pem -cert client.pem -key client.key \
  -d '{"user_id": "123"}' \
  my-service:50051 mycompany.user.v1.UserService/GetUser

# Stream
grpcurl -plaintext -d '{"query": "error"}' \
  localhost:50051 mycompany.log.v1.LogService/StreamLogs
```

### Testing with buf

```bash
# Lint proto files
buf lint

# Check for breaking changes
buf breaking --against '.git#branch=main'

# Generate code
buf generate
```

---

## Observability

### Logging

```rust
// Rust: Structured logging with tracing
use tracing::{info, error, instrument};

#[instrument(skip(self))]
async fn get_user(
    &self,
    request: Request<GetUserRequest>,
) -> Result<Response<GetUserResponse>, Status> {
    let user_id = &request.get_ref().user_id;

    info!(user_id = %user_id, "Getting user");

    match self.repo.get_user(user_id).await {
        Ok(user) => {
            info!(user_id = %user_id, "User found");
            Ok(Response::new(GetUserResponse { user: Some(user) }))
        }
        Err(e) => {
            error!(user_id = %user_id, error = ?e, "Failed to get user");
            Err(Status::internal("Failed to get user"))
        }
    }
}
```

### Metrics (Prometheus)

```rust
// Rust: tonic with prometheus metrics
use prometheus::{Counter, Histogram, register_counter, register_histogram};

lazy_static! {
    static ref REQUESTS_TOTAL: Counter = register_counter!(
        "grpc_requests_total",
        "Total gRPC requests"
    ).unwrap();

    static ref REQUEST_DURATION: Histogram = register_histogram!(
        "grpc_request_duration_seconds",
        "gRPC request duration"
    ).unwrap();
}

// In interceptor
fn metrics_interceptor<T>(req: Request<T>) -> Result<Request<T>, Status> {
    REQUESTS_TOTAL.inc();
    // Start timer, record duration on response
    Ok(req)
}
```

### Distributed Tracing (OpenTelemetry)

```rust
// Rust: tonic with OpenTelemetry
use opentelemetry::global;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::prelude::*;

fn init_tracing() {
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("user-service")
        .install_batch(opentelemetry::runtime::Tokio)
        .unwrap();

    let telemetry = OpenTelemetryLayer::new(tracer);

    tracing_subscriber::registry()
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

// Propagate trace context in interceptor
fn trace_interceptor<T>(mut req: Request<T>) -> Result<Request<T>, Status> {
    // Extract trace context from incoming request
    // Inject into outgoing requests
    Ok(req)
}
```

### Health Checks

```protobuf
// Standard health check protocol
syntax = "proto3";

package grpc.health.v1;

service Health {
  rpc Check(HealthCheckRequest) returns (HealthCheckResponse);
  rpc Watch(HealthCheckRequest) returns (stream HealthCheckResponse);
}

message HealthCheckRequest {
  string service = 1;
}

message HealthCheckResponse {
  enum ServingStatus {
    UNKNOWN = 0;
    SERVING = 1;
    NOT_SERVING = 2;
    SERVICE_UNKNOWN = 3;
  }
  ServingStatus status = 1;
}
```

```rust
// Rust (tonic): Built-in health service
use tonic_health::server::health_reporter;

let (mut health_reporter, health_service) = health_reporter();

// Set service health
health_reporter
    .set_serving::<UserServiceServer<MyUserService>>()
    .await;

Server::builder()
    .add_service(health_service)
    .add_service(UserServiceServer::new(my_service))
    .serve(addr)
    .await?;
```

---

## Framework Examples

### NestJS gRPC Implementation

NestJS provides excellent gRPC support through `@nestjs/microservices`.

#### Installation

```bash
npm install @nestjs/microservices @grpc/grpc-js @grpc/proto-loader
```

#### Proto File

```protobuf
// proto/user.proto
syntax = "proto3";

package user;

service UserService {
  rpc GetUser(GetUserRequest) returns (User);
  rpc ListUsers(ListUsersRequest) returns (ListUsersResponse);
  rpc CreateUser(CreateUserRequest) returns (User);
  rpc UpdateUser(UpdateUserRequest) returns (User);
  rpc DeleteUser(DeleteUserRequest) returns (DeleteUserResponse);
  rpc StreamUsers(StreamUsersRequest) returns (stream User);
}

message User {
  string id = 1;
  string name = 2;
  string email = 3;
  int64 created_at = 4;
}

message GetUserRequest {
  string id = 1;
}

message ListUsersRequest {
  int32 page_size = 1;
  string page_token = 2;
}

message ListUsersResponse {
  repeated User users = 1;
  string next_page_token = 2;
}

message CreateUserRequest {
  string name = 1;
  string email = 2;
}

message UpdateUserRequest {
  string id = 1;
  string name = 2;
  string email = 3;
}

message DeleteUserRequest {
  string id = 1;
}

message DeleteUserResponse {
  bool success = 1;
}

message StreamUsersRequest {
  string filter = 1;
}
```

#### Server Setup (main.ts)

```typescript
// main.ts
import { NestFactory } from '@nestjs/core';
import { MicroserviceOptions, Transport } from '@nestjs/microservices';
import { join } from 'path';
import { AppModule } from './app.module';

async function bootstrap() {
  const app = await NestFactory.createMicroservice<MicroserviceOptions>(
    AppModule,
    {
      transport: Transport.GRPC,
      options: {
        package: 'user',
        protoPath: join(__dirname, '../proto/user.proto'),
        url: '0.0.0.0:50051',
        // Security: Set message size limits
        maxReceiveMessageLength: 4 * 1024 * 1024, // 4MB
        maxSendMessageLength: 4 * 1024 * 1024,
        // Enable keepalive
        keepalive: {
          keepaliveTimeMs: 10000,
          keepaliveTimeoutMs: 5000,
          keepalivePermitWithoutCalls: 1,
        },
      },
    },
  );

  await app.listen();
  console.log('gRPC server running on port 50051');
}

bootstrap();
```

#### Hybrid Server (HTTP + gRPC)

```typescript
// main.ts - Hybrid setup
import { NestFactory } from '@nestjs/core';
import { MicroserviceOptions, Transport } from '@nestjs/microservices';
import { join } from 'path';
import { AppModule } from './app.module';

async function bootstrap() {
  // HTTP server
  const app = await NestFactory.create(AppModule);

  // Add gRPC microservice
  app.connectMicroservice<MicroserviceOptions>({
    transport: Transport.GRPC,
    options: {
      package: 'user',
      protoPath: join(__dirname, '../proto/user.proto'),
      url: '0.0.0.0:50051',
    },
  });

  await app.startAllMicroservices();
  await app.listen(3000);

  console.log('HTTP server on port 3000, gRPC on port 50051');
}

bootstrap();
```

#### Service Implementation

```typescript
// user.controller.ts
import { Controller } from '@nestjs/common';
import { GrpcMethod, GrpcStreamMethod } from '@nestjs/microservices';
import { Observable, Subject } from 'rxjs';
import { Metadata, ServerUnaryCall } from '@grpc/grpc-js';
import { RpcException } from '@nestjs/microservices';
import { status } from '@grpc/grpc-js';

interface User {
  id: string;
  name: string;
  email: string;
  createdAt: number;
}

interface GetUserRequest {
  id: string;
}

interface CreateUserRequest {
  name: string;
  email: string;
}

interface ListUsersRequest {
  pageSize: number;
  pageToken: string;
}

interface ListUsersResponse {
  users: User[];
  nextPageToken: string;
}

interface StreamUsersRequest {
  filter: string;
}

@Controller()
export class UserController {
  private users: Map<string, User> = new Map();

  // Unary RPC
  @GrpcMethod('UserService', 'GetUser')
  getUser(
    data: GetUserRequest,
    metadata: Metadata,
    call: ServerUnaryCall<GetUserRequest, User>,
  ): User {
    // Input validation
    if (!data.id || data.id.trim() === '') {
      throw new RpcException({
        code: status.INVALID_ARGUMENT,
        message: 'User ID is required',
      });
    }

    const user = this.users.get(data.id);

    if (!user) {
      throw new RpcException({
        code: status.NOT_FOUND,
        message: `User with ID ${data.id} not found`,
      });
    }

    return user;
  }

  // Unary RPC - Create
  @GrpcMethod('UserService', 'CreateUser')
  createUser(data: CreateUserRequest): User {
    // Validation
    if (!data.name || data.name.trim() === '') {
      throw new RpcException({
        code: status.INVALID_ARGUMENT,
        message: 'Name is required',
      });
    }

    if (!data.email || !this.isValidEmail(data.email)) {
      throw new RpcException({
        code: status.INVALID_ARGUMENT,
        message: 'Valid email is required',
      });
    }

    // Check for duplicate email
    for (const user of this.users.values()) {
      if (user.email === data.email) {
        throw new RpcException({
          code: status.ALREADY_EXISTS,
          message: 'User with this email already exists',
        });
      }
    }

    const user: User = {
      id: crypto.randomUUID(),
      name: data.name,
      email: data.email,
      createdAt: Date.now(),
    };

    this.users.set(user.id, user);
    return user;
  }

  // Unary RPC - List with pagination
  @GrpcMethod('UserService', 'ListUsers')
  listUsers(data: ListUsersRequest): ListUsersResponse {
    const pageSize = Math.min(data.pageSize || 10, 100); // Max 100
    const allUsers = Array.from(this.users.values());

    let startIndex = 0;
    if (data.pageToken) {
      startIndex = parseInt(data.pageToken, 10) || 0;
    }

    const users = allUsers.slice(startIndex, startIndex + pageSize);
    const nextIndex = startIndex + pageSize;

    return {
      users,
      nextPageToken: nextIndex < allUsers.length ? String(nextIndex) : '',
    };
  }

  // Server streaming RPC
  @GrpcStreamMethod('UserService', 'StreamUsers')
  streamUsers(data: Observable<StreamUsersRequest>): Observable<User> {
    const subject = new Subject<User>();

    const onNext = (request: StreamUsersRequest) => {
      // Stream all users matching filter
      for (const user of this.users.values()) {
        if (!request.filter || user.name.includes(request.filter)) {
          subject.next(user);
        }
      }
    };

    const onComplete = () => {
      subject.complete();
    };

    data.subscribe({
      next: onNext,
      complete: onComplete,
    });

    return subject.asObservable();
  }

  private isValidEmail(email: string): boolean {
    return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
  }
}
```

#### gRPC Client in NestJS

```typescript
// user-client.service.ts
import { Injectable, OnModuleInit } from '@nestjs/common';
import { Client, ClientGrpc, Transport } from '@nestjs/microservices';
import { join } from 'path';
import { Observable, lastValueFrom, timeout } from 'rxjs';

interface UserServiceClient {
  getUser(data: { id: string }): Observable<User>;
  listUsers(data: { pageSize: number; pageToken: string }): Observable<ListUsersResponse>;
  createUser(data: { name: string; email: string }): Observable<User>;
  streamUsers(data: Observable<{ filter: string }>): Observable<User>;
}

@Injectable()
export class UserClientService implements OnModuleInit {
  @Client({
    transport: Transport.GRPC,
    options: {
      package: 'user',
      protoPath: join(__dirname, '../proto/user.proto'),
      url: 'localhost:50051',
      // Connection options
      channelOptions: {
        'grpc.keepalive_time_ms': 10000,
        'grpc.keepalive_timeout_ms': 5000,
      },
    },
  })
  private client: ClientGrpc;

  private userService: UserServiceClient;

  onModuleInit() {
    this.userService = this.client.getService<UserServiceClient>('UserService');
  }

  // Always set timeouts!
  async getUser(id: string): Promise<User> {
    return lastValueFrom(
      this.userService.getUser({ id }).pipe(
        timeout(5000), // 5 second timeout
      ),
    );
  }

  async createUser(name: string, email: string): Promise<User> {
    return lastValueFrom(
      this.userService.createUser({ name, email }).pipe(
        timeout(5000),
      ),
    );
  }

  // Streaming example
  streamUsers(filter: string): Observable<User> {
    const request$ = new Observable<{ filter: string }>((subscriber) => {
      subscriber.next({ filter });
      subscriber.complete();
    });

    return this.userService.streamUsers(request$);
  }
}
```

#### Error Handling Interceptor

```typescript
// grpc-exception.filter.ts
import { Catch, RpcExceptionFilter, ArgumentsHost } from '@nestjs/common';
import { Observable, throwError } from 'rxjs';
import { RpcException } from '@nestjs/microservices';
import { status } from '@grpc/grpc-js';

@Catch(RpcException)
export class GrpcExceptionFilter implements RpcExceptionFilter<RpcException> {
  catch(exception: RpcException, host: ArgumentsHost): Observable<any> {
    const error = exception.getError() as { code: number; message: string };

    // Log internally
    console.error('gRPC Error:', {
      code: error.code,
      message: error.message,
      timestamp: new Date().toISOString(),
    });

    return throwError(() => exception);
  }
}

// Global error handler for unexpected errors
@Catch()
export class AllExceptionsFilter implements RpcExceptionFilter {
  catch(exception: unknown, host: ArgumentsHost): Observable<any> {
    console.error('Unexpected error:', exception);

    // Don't leak internal details
    return throwError(
      () =>
        new RpcException({
          code: status.INTERNAL,
          message: 'Internal server error',
        }),
    );
  }
}
```

#### Authentication Guard

```typescript
// grpc-auth.guard.ts
import { Injectable, CanActivate, ExecutionContext } from '@nestjs/common';
import { RpcException } from '@nestjs/microservices';
import { status } from '@grpc/grpc-js';

@Injectable()
export class GrpcAuthGuard implements CanActivate {
  canActivate(context: ExecutionContext): boolean {
    const metadata = context.switchToRpc().getContext();

    // Get authorization header from metadata
    const authHeader = metadata.get('authorization')?.[0];

    if (!authHeader) {
      throw new RpcException({
        code: status.UNAUTHENTICATED,
        message: 'Missing authorization header',
      });
    }

    if (!authHeader.startsWith('Bearer ')) {
      throw new RpcException({
        code: status.UNAUTHENTICATED,
        message: 'Invalid authorization format',
      });
    }

    const token = authHeader.slice(7);

    try {
      // Validate JWT token
      const payload = this.validateToken(token);

      // Attach user to context
      metadata.user = payload;
      return true;
    } catch (error) {
      throw new RpcException({
        code: status.UNAUTHENTICATED,
        message: 'Invalid or expired token',
      });
    }
  }

  private validateToken(token: string): any {
    // Implement JWT validation
    // return jwt.verify(token, process.env.JWT_SECRET);
    return { userId: '123', roles: ['user'] };
  }
}
```

#### Health Check

```typescript
// health.controller.ts
import { Controller } from '@nestjs/common';
import { GrpcMethod } from '@nestjs/microservices';

interface HealthCheckRequest {
  service: string;
}

interface HealthCheckResponse {
  status: 'UNKNOWN' | 'SERVING' | 'NOT_SERVING' | 'SERVICE_UNKNOWN';
}

@Controller()
export class HealthController {
  @GrpcMethod('Health', 'Check')
  check(data: HealthCheckRequest): HealthCheckResponse {
    // Check service health
    const isHealthy = this.checkDependencies();

    return {
      status: isHealthy ? 'SERVING' : 'NOT_SERVING',
    };
  }

  private checkDependencies(): boolean {
    // Check database, external services, etc.
    return true;
  }
}
```

---

### Hono with ConnectRPC

Hono doesn't have native gRPC support, but you can use **ConnectRPC** which provides gRPC-compatible APIs over HTTP/1.1 and HTTP/2.

#### Installation

```bash
npm install hono @connectrpc/connect @connectrpc/connect-node @bufbuild/protobuf
npm install -D @bufbuild/buf @bufbuild/protoc-gen-es @connectrpc/protoc-gen-connect-es
```

#### buf.gen.yaml for ConnectRPC

```yaml
# buf.gen.yaml
version: v1
plugins:
  # Generate TypeScript message types
  - plugin: buf.build/bufbuild/es
    out: gen
    opt: target=ts

  # Generate ConnectRPC service stubs
  - plugin: buf.build/connectrpc/es
    out: gen
    opt: target=ts
```

#### Proto File

```protobuf
// proto/user/v1/user.proto
syntax = "proto3";

package user.v1;

service UserService {
  rpc GetUser(GetUserRequest) returns (GetUserResponse);
  rpc CreateUser(CreateUserRequest) returns (CreateUserResponse);
  rpc ListUsers(ListUsersRequest) returns (ListUsersResponse);
  rpc StreamUsers(StreamUsersRequest) returns (stream User);
}

message User {
  string id = 1;
  string name = 2;
  string email = 3;
  int64 created_at = 4;
}

message GetUserRequest {
  string id = 1;
}

message GetUserResponse {
  User user = 1;
}

message CreateUserRequest {
  string name = 1;
  string email = 2;
}

message CreateUserResponse {
  User user = 1;
}

message ListUsersRequest {
  int32 page_size = 1;
  string page_token = 2;
}

message ListUsersResponse {
  repeated User users = 1;
  string next_page_token = 2;
}

message StreamUsersRequest {
  string filter = 1;
}
```

#### Generate Code

```bash
npx buf generate proto
```

#### ConnectRPC Service Implementation

```typescript
// services/user-service.ts
import { ConnectRouter } from '@connectrpc/connect';
import { UserService } from '../gen/user/v1/user_connect';
import {
  GetUserRequest,
  GetUserResponse,
  CreateUserRequest,
  CreateUserResponse,
  ListUsersRequest,
  ListUsersResponse,
  StreamUsersRequest,
  User,
} from '../gen/user/v1/user_pb';
import { Code, ConnectError } from '@connectrpc/connect';

// In-memory store (use a real database in production)
const users = new Map<string, User>();

export default (router: ConnectRouter) =>
  router.service(UserService, {
    // Unary RPC
    async getUser(request: GetUserRequest): Promise<GetUserResponse> {
      // Input validation
      if (!request.id || request.id.trim() === '') {
        throw new ConnectError('User ID is required', Code.InvalidArgument);
      }

      const user = users.get(request.id);

      if (!user) {
        throw new ConnectError(
          `User with ID ${request.id} not found`,
          Code.NotFound,
        );
      }

      return new GetUserResponse({ user });
    },

    // Create user
    async createUser(request: CreateUserRequest): Promise<CreateUserResponse> {
      // Validation
      if (!request.name || request.name.trim() === '') {
        throw new ConnectError('Name is required', Code.InvalidArgument);
      }

      if (!request.email || !isValidEmail(request.email)) {
        throw new ConnectError('Valid email is required', Code.InvalidArgument);
      }

      // Check for duplicate
      for (const user of users.values()) {
        if (user.email === request.email) {
          throw new ConnectError(
            'User with this email already exists',
            Code.AlreadyExists,
          );
        }
      }

      const user = new User({
        id: crypto.randomUUID(),
        name: request.name,
        email: request.email,
        createdAt: BigInt(Date.now()),
      });

      users.set(user.id, user);

      return new CreateUserResponse({ user });
    },

    // List with pagination
    async listUsers(request: ListUsersRequest): Promise<ListUsersResponse> {
      const pageSize = Math.min(request.pageSize || 10, 100);
      const allUsers = Array.from(users.values());

      let startIndex = 0;
      if (request.pageToken) {
        startIndex = parseInt(request.pageToken, 10) || 0;
      }

      const pageUsers = allUsers.slice(startIndex, startIndex + pageSize);
      const nextIndex = startIndex + pageSize;

      return new ListUsersResponse({
        users: pageUsers,
        nextPageToken: nextIndex < allUsers.length ? String(nextIndex) : '',
      });
    },

    // Server streaming
    async *streamUsers(request: StreamUsersRequest): AsyncIterable<User> {
      for (const user of users.values()) {
        if (!request.filter || user.name.includes(request.filter)) {
          yield user;
          // Simulate delay for streaming effect
          await new Promise((resolve) => setTimeout(resolve, 100));
        }
      }
    },
  });

function isValidEmail(email: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
}
```

#### Hono Server with ConnectRPC

```typescript
// server.ts
import { Hono } from 'hono';
import { serve } from '@hono/node-server';
import { cors } from 'hono/cors';
import { logger } from 'hono/logger';
import { connectNodeAdapter } from '@connectrpc/connect-node';
import { ConnectRouter } from '@connectrpc/connect';
import userService from './services/user-service';

// Create ConnectRPC router
const connectRouter = (router: ConnectRouter) => {
  userService(router);
};

// Create Hono app
const app = new Hono();

// Middleware
app.use('*', logger());
app.use(
  '*',
  cors({
    origin: ['http://localhost:3000'],
    allowHeaders: [
      'Content-Type',
      'Connect-Protocol-Version',
      'Connect-Timeout-Ms',
      'Authorization',
      'X-Request-Id',
    ],
    allowMethods: ['POST', 'GET', 'OPTIONS'],
    exposeHeaders: ['Connect-Content-Encoding', 'grpc-status', 'grpc-message'],
  }),
);

// Health check endpoint
app.get('/health', (c) => c.json({ status: 'ok' }));

// ConnectRPC handler
const connectHandler = connectNodeAdapter({
  routes: connectRouter,
  // Security options
  maxTimeoutMs: 30000, // 30 second max timeout
  // Accept both Connect and gRPC-Web protocols
  acceptCompression: [],
});

// Mount ConnectRPC at root (it handles its own routing)
app.all('/*', async (c) => {
  const response = await connectHandler(c.req.raw);
  return response;
});

// Start server
const port = Number(process.env.PORT) || 3000;

serve(
  {
    fetch: app.fetch,
    port,
  },
  (info) => {
    console.log(`Server running at http://localhost:${info.port}`);
    console.log(`ConnectRPC services available at http://localhost:${info.port}`);
  },
);
```

#### Hono Middleware for Authentication

```typescript
// middleware/auth.ts
import { createMiddleware } from 'hono/factory';
import { Code, ConnectError } from '@connectrpc/connect';

interface AuthContext {
  userId: string;
  roles: string[];
}

declare module 'hono' {
  interface ContextVariableMap {
    auth: AuthContext;
  }
}

export const authMiddleware = createMiddleware(async (c, next) => {
  const authHeader = c.req.header('Authorization');

  if (!authHeader) {
    throw new ConnectError('Missing authorization header', Code.Unauthenticated);
  }

  if (!authHeader.startsWith('Bearer ')) {
    throw new ConnectError('Invalid authorization format', Code.Unauthenticated);
  }

  const token = authHeader.slice(7);

  try {
    // Validate token (implement your JWT validation)
    const payload = validateToken(token);
    c.set('auth', payload);
    await next();
  } catch (error) {
    throw new ConnectError('Invalid or expired token', Code.Unauthenticated);
  }
});

function validateToken(token: string): AuthContext {
  // Implement JWT validation
  return { userId: '123', roles: ['user'] };
}
```

#### ConnectRPC Interceptors

```typescript
// interceptors/logging.ts
import { Interceptor } from '@connectrpc/connect';

export const loggingInterceptor: Interceptor = (next) => async (req) => {
  const start = Date.now();
  const requestId = crypto.randomUUID();

  console.log(`[${requestId}] --> ${req.method.name}`);

  try {
    const response = await next(req);
    const duration = Date.now() - start;
    console.log(`[${requestId}] <-- ${req.method.name} (${duration}ms)`);
    return response;
  } catch (error) {
    const duration = Date.now() - start;
    console.error(`[${requestId}] <-- ${req.method.name} ERROR (${duration}ms)`, error);
    throw error;
  }
};

// interceptors/timeout.ts
export const timeoutInterceptor: Interceptor = (next) => async (req) => {
  // Set default timeout if not specified
  if (!req.timeoutMs) {
    req.timeoutMs = 5000; // 5 seconds default
  }
  return next(req);
};
```

#### ConnectRPC Client

```typescript
// client.ts
import { createClient } from '@connectrpc/connect';
import { createConnectTransport } from '@connectrpc/connect-node';
import { UserService } from './gen/user/v1/user_connect';
import { loggingInterceptor, timeoutInterceptor } from './interceptors';

// Create transport with interceptors
const transport = createConnectTransport({
  baseUrl: 'http://localhost:3000',
  httpVersion: '2', // Use HTTP/2
  interceptors: [loggingInterceptor, timeoutInterceptor],
});

// Create typed client
const client = createClient(UserService, transport);

// Usage examples
async function main() {
  try {
    // Create user
    const createResponse = await client.createUser({
      name: 'John Doe',
      email: 'john@example.com',
    });
    console.log('Created user:', createResponse.user);

    // Get user
    const getResponse = await client.getUser({
      id: createResponse.user!.id,
    });
    console.log('Got user:', getResponse.user);

    // List users
    const listResponse = await client.listUsers({
      pageSize: 10,
    });
    console.log('Users:', listResponse.users);

    // Stream users
    console.log('Streaming users:');
    for await (const user of client.streamUsers({ filter: '' })) {
      console.log('  -', user.name);
    }
  } catch (error) {
    console.error('Error:', error);
  }
}

main();
```

#### Browser Client (React/Vue/etc.)

```typescript
// browser-client.ts
import { createClient } from '@connectrpc/connect';
import { createConnectTransport } from '@connectrpc/connect-web';
import { UserService } from './gen/user/v1/user_connect';

// Browser transport (uses fetch)
const transport = createConnectTransport({
  baseUrl: 'http://localhost:3000',
});

export const userClient = createClient(UserService, transport);

// React hook example
import { useQuery, useMutation } from '@tanstack/react-query';

export function useUser(id: string) {
  return useQuery({
    queryKey: ['user', id],
    queryFn: () => userClient.getUser({ id }),
  });
}

export function useCreateUser() {
  return useMutation({
    mutationFn: (data: { name: string; email: string }) =>
      userClient.createUser(data),
  });
}
```

### Framework Comparison

| Feature | NestJS gRPC | Hono + ConnectRPC |
|---------|-------------|-------------------|
| Protocol | Native gRPC (HTTP/2) | Connect, gRPC-Web, gRPC |
| Browser support | Requires grpc-web proxy | Native browser support |
| Streaming | Full bidirectional | Server streaming (browser), full (Node) |
| Type safety | Manual interfaces | Generated from proto |
| Bundle size | Heavy (@grpc/grpc-js) | Lighter (ConnectRPC) |
| Learning curve | NestJS patterns | Simpler, more explicit |
| Best for | Backend microservices | Web-first APIs, edge |

---

## Common Pitfalls

### 1. Not Setting Deadlines

```rust
// BAD: No deadline - can hang forever
client.get_user(request).await?;

// GOOD: Always set deadline
let mut request = Request::new(GetUserRequest { user_id: "123".into() });
request.set_timeout(Duration::from_secs(5));
client.get_user(request).await?;
```

### 2. Creating Connections Per Request

```rust
// BAD: New connection every call
async fn call() {
    let channel = Channel::from_static("http://localhost:50051")
        .connect()
        .await?;
    let client = UserServiceClient::new(channel);
    client.get_user(request).await?;
}

// GOOD: Reuse channel
static CHANNEL: Lazy<Channel> = Lazy::new(|| {
    Channel::from_static("http://localhost:50051").connect_lazy()
});

async fn call() {
    let client = UserServiceClient::new(CHANNEL.clone());
    client.get_user(request).await?;
}
```

### 3. Ignoring Errors in Streams

```rust
// BAD: Silent failures
while let Some(msg) = stream.message().await? {
    process(msg);  // What if this fails?
}

// GOOD: Handle errors
while let Some(result) = stream.message().await.transpose() {
    match result {
        Ok(msg) => {
            if let Err(e) = process(msg) {
                error!("Failed to process message: {}", e);
            }
        }
        Err(e) => {
            error!("Stream error: {}", e);
            break;
        }
    }
}
```

### 4. Large Messages Without Streaming

```rust
// BAD: Sending 100MB in one message
rpc GetAllUsers(Empty) returns (AllUsersResponse);  // Response has 1M users

// GOOD: Stream large responses
rpc ListAllUsers(ListAllUsersRequest) returns (stream User);
```

### 5. Missing Input Validation

```rust
// BAD: Trust client input
async fn get_user(&self, request: Request<GetUserRequest>) -> ... {
    let user_id = request.into_inner().user_id;
    self.db.query(&format!("SELECT * FROM users WHERE id = {}", user_id))  // SQL injection!
}

// GOOD: Validate and sanitize
async fn get_user(&self, request: Request<GetUserRequest>) -> ... {
    let user_id = request.into_inner().user_id;

    // Validate
    if user_id.is_empty() {
        return Err(Status::invalid_argument("user_id is required"));
    }
    if user_id.len() > 36 {
        return Err(Status::invalid_argument("user_id too long"));
    }

    // Use parameterized query
    self.db.query("SELECT * FROM users WHERE id = $1", &[&user_id])
}
```

### 6. Blocking in Async Context

```rust
// BAD: Blocking call in async
async fn process(&self, request: Request<ProcessRequest>) -> ... {
    let data = request.into_inner().data;
    let result = expensive_sync_computation(&data);  // Blocks the runtime!
    Ok(Response::new(ProcessResponse { result }))
}

// GOOD: Spawn blocking task
async fn process(&self, request: Request<ProcessRequest>) -> ... {
    let data = request.into_inner().data;
    let result = tokio::task::spawn_blocking(move || {
        expensive_sync_computation(&data)
    }).await?;
    Ok(Response::new(ProcessResponse { result }))
}
```

---

## Resources

### Official Documentation

- [gRPC Documentation](https://grpc.io/docs/)
- [Protocol Buffers Guide](https://protobuf.dev/programming-guides/proto3/)
- [Google API Design Guide](https://cloud.google.com/apis/design)
- [Google AIP (API Improvement Proposals)](https://google.aip.dev/)
- [Buf Documentation](https://buf.build/docs/)

### Videos

- [gRPC Crash Course - Modes, Examples, Pros & Cons](https://www.youtube.com/watch?v=Yw4rkaTc0f8) - Hussein Nasser
- [Protocol Buffers Crash Course](https://www.youtube.com/watch?v=46O73On0gyI) - Traversy Media
- [gRPC vs REST - Performance Comparison](https://www.youtube.com/watch?v=u4LWEXR6t9I) - Tech Primers
- [Building High Performance APIs with gRPC](https://www.youtube.com/watch?v=MaP61eiL7Ug) - InfoQ
- [gRPC and Protocol Buffers in Rust](https://www.youtube.com/watch?v=C7WxYfmF-uI) - Let's Get Rusty
- [Microservices with gRPC](https://www.youtube.com/watch?v=hVrwuMnCtok) - TechWorld with Nana

### Articles & Tutorials

- [gRPC Best Practices](https://kreya.app/blog/grpc-best-practices/)
- [Practical Guide to gRPC](https://www.cncf.io/blog/2021/07/19/think-grpc-when-you-are-architecting-modern-microservices/)
- [gRPC Load Balancing](https://grpc.io/blog/grpc-load-balancing/)
- [Error Handling in gRPC](https://avi.im/grpc-errors/)
- [gRPC Authentication Guide](https://grpc.io/docs/guides/auth/)

### Rust-Specific Resources

- [tonic - gRPC for Rust](https://github.com/hyperium/tonic)
- [tonic Examples](https://github.com/hyperium/tonic/tree/master/examples)
- [prost - Protocol Buffers for Rust](https://github.com/tokio-rs/prost)
- [Building gRPC APIs with Rust](https://dev.to/anshulxyz/building-grpc-apis-with-rust-using-tonic-36nh)

### Go-Specific Resources

- [gRPC-Go](https://github.com/grpc/grpc-go)
- [gRPC-Go Examples](https://github.com/grpc/grpc-go/tree/master/examples)
- [Go gRPC Middleware](https://github.com/grpc-ecosystem/go-grpc-middleware)

### NestJS-Specific Resources

- [NestJS gRPC Documentation](https://docs.nestjs.com/microservices/grpc)
- [NestJS Microservices](https://docs.nestjs.com/microservices/basics)
- [@grpc/grpc-js](https://www.npmjs.com/package/@grpc/grpc-js)

### Hono / ConnectRPC Resources

- [ConnectRPC](https://connectrpc.com/) - Modern gRPC-compatible protocol for web
- [Hono](https://hono.dev/) - Lightweight web framework
- [@connectrpc/connect](https://www.npmjs.com/package/@connectrpc/connect)

### Tools

| Tool | Purpose |
|------|---------|
| [buf](https://buf.build/) | Modern protobuf tooling (lint, breaking changes, generate) |
| [grpcurl](https://github.com/fullstorydev/grpcurl) | curl for gRPC |
| [grpcui](https://github.com/fullstorydev/grpcui) | Web UI for gRPC |
| [ghz](https://ghz.sh/) | gRPC load testing |
| [evans](https://github.com/ktr0731/evans) | gRPC REPL |
| [BloomRPC](https://github.com/bloomrpc/bloomrpc) | gRPC GUI client (archived, use Postman) |
| [Postman](https://www.postman.com/) | API testing with gRPC support |
| [Kreya](https://kreya.app/) | gRPC GUI client |

### Security Resources

- [OWASP API Security Top 10](https://owasp.org/API-Security/)
- [gRPC Security Audit Results](https://github.com/grpc/grpc/blob/master/doc/security_audit.md)
- [mTLS with gRPC](https://itnext.io/practical-guide-to-securing-grpc-connections-with-tls-golang-9e8d1c4a8e2f)

### Books

- *gRPC: Up and Running* by Kasun Indrasiri & Danesh Kuruppu (O'Reilly)
- *Building Microservices* by Sam Newman (covers gRPC in distributed systems context)

---

## Quick Reference

### Proto Style Checklist

- [ ] Package name: `company.service.v1`
- [ ] Service suffix: `UserService` not `Users`
- [ ] RPC names: `GetUser`, `ListUsers`, `CreateUser`
- [ ] Message names: `PascalCase`
- [ ] Field names: `snake_case`
- [ ] Enum values: `ENUM_NAME_VALUE` with `UNSPECIFIED = 0`
- [ ] Dedicated request/response messages
- [ ] Version in package path

### Status Code Quick Guide

| Scenario | Code |
|----------|------|
| Success | OK |
| Bad input | INVALID_ARGUMENT |
| Not found | NOT_FOUND |
| Already exists | ALREADY_EXISTS |
| No auth token | UNAUTHENTICATED |
| Not authorized | PERMISSION_DENIED |
| Rate limited | RESOURCE_EXHAUSTED |
| Timeout | DEADLINE_EXCEEDED |
| Bug/crash | INTERNAL |
| Service down | UNAVAILABLE |

### buf.yaml Template

```yaml
version: v1
deps:
  - buf.build/googleapis/googleapis
breaking:
  use:
    - FILE
lint:
  use:
    - DEFAULT
  except:
    - PACKAGE_VERSION_SUFFIX
```

### buf.gen.yaml Template

```yaml
version: v1
plugins:
  # Rust
  - plugin: buf.build/community/neoeinstein-prost
    out: gen/rust
  - plugin: buf.build/community/neoeinstein-tonic
    out: gen/rust

  # Go
  - plugin: buf.build/protocolbuffers/go
    out: gen/go
    opt: paths=source_relative
  - plugin: buf.build/grpc/go
    out: gen/go
    opt: paths=source_relative
```
