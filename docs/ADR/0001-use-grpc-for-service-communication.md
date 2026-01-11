# ADR-0001: Use gRPC for Service Communication

## Status

Accepted

## Date

2024-01-15

## Context

As the platform grows, we need a reliable, efficient way for services to communicate. Key requirements:

- Low latency for synchronous calls
- Strong typing for API contracts
- Support for streaming (large result sets, real-time updates)
- Polyglot support (Rust backend, TypeScript frontend)
- Good tooling and ecosystem

## Decision

Use **gRPC with Protocol Buffers** for internal service-to-service communication via the Tonic framework in Rust.

### Implementation

- Proto definitions in `manifests/grpc/proto/apps/v1/`
- Rust codegen via `prost` and `tonic-build`
- TypeScript codegen via `@connectrpc/connect` for web clients
- Services: Tasks (50051), Vector (50052)

### API Gateway Pattern

- `zerg-api` acts as HTTP gateway for external clients
- Converts HTTP/JSON to gRPC calls
- Lazy client connections (services don't need to be up at startup)

## Consequences

### Positive

- **Performance**: Binary protocol, HTTP/2 multiplexing
- **Type Safety**: Generated clients catch errors at compile time
- **Streaming**: Native support for server/client/bidirectional streaming
- **Evolution**: Protobuf handles schema evolution gracefully
- **Tooling**: grpcurl, gRPC health checks, reflection

### Negative

- **Learning Curve**: Team needs to understand Protocol Buffers
- **Debugging**: Binary format harder to inspect than JSON
- **Browser Support**: Requires gRPC-Web or Connect protocol proxy

### Risks

- **Single Point of Failure**: If gRPC server is down, API fails
  - Mitigation: Circuit breakers, fallback endpoints
- **Proto Evolution**: Breaking changes require coordination
  - Mitigation: Versioned packages, deprecation process

## Alternatives Considered

### REST/JSON

- Simpler, more widely understood
- Rejected: Higher latency, no streaming, weaker contracts

### GraphQL

- Good for complex query patterns
- Rejected: Overkill for service-to-service, not suited for streaming

### Message Queue (NATS/Kafka)

- Better for async communication
- Rejected: Not suitable for request/response patterns (used separately for async)

## References

- [gRPC Documentation](https://grpc.io/)
- [Tonic Rust Framework](https://github.com/hyperium/tonic)
- [Connect Protocol](https://connect.build/)
- [docs/grpc.md](../grpc.md)
