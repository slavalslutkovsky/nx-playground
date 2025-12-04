# Zerg API - TODO

This document tracks future improvements and enhancements for the Zerg API.

## Production Readiness

### High Priority

- [ ] **Add shutdown metrics**
  - Track shutdown duration
  - Monitor cleanup success/failure rates
  - Alert on timeouts
  - Location: Add to `src/main.rs` shutdown handler
  - Tools: Prometheus metrics, OpenTelemetry

- [ ] **Configure Kubernetes manifests**
  - Set `terminationGracePeriodSeconds: 60` (must be > 30s shutdown timeout)
  - Add `preStop` hook with 5-10s sleep
  - Configure readiness probe to fail during shutdown
  - Update liveness probe settings
  - Location: `manifests/k8s/deployment.yaml` (create if doesn't exist)

- [ ] **Load test shutdown behavior**
  - Test shutdown under realistic traffic
  - Verify no 502/503 errors during rolling updates
  - Measure connection cleanup time under load
  - Test with 100+ concurrent requests
  - Tools: k6, vegeta, or wrk

### Medium Priority

- [ ] **Add shutdown timeout alerts**
  - Alert when cleanup exceeds 30s
  - Alert on database connection cleanup failures
  - Alert on Redis connection issues during shutdown
  - Integration: Datadog, Grafana, or CloudWatch

- [ ] **Implement health check enhancements**
  - Make `/ready` return 503 during shutdown
  - Add shutdown state to health responses
  - Add detailed connection pool status
  - Location: `src/main.rs:79-108` (ready handler)

- [ ] **Database connection pool optimization**
  - Review connection pool settings (currently max: 100)
  - Add connection pool metrics
  - Tune based on load testing results
  - Consider separate read/write pools

- [ ] **Chaos engineering tests**
  - Test random pod kills during traffic
  - Test database connection loss during shutdown
  - Test Redis unavailable during shutdown
  - Test rapid successive shutdown signals
  - Tools: Chaos Mesh, Litmus

## API Documentation

- [ ] **Migrate to axum-helpers OpenAPI integration**
  - Use `create_router()` with OpenAPI docs
  - Add utoipa annotations to handlers
  - Integrate Swagger UI, ReDoc, Scalar
  - Location: Update `src/main.rs` router setup

- [ ] **Add API versioning**
  - Implement `/api/v1/` prefix
  - Version all endpoints
  - Document versioning strategy

- [ ] **Add request/response examples**
  - Document all endpoints with examples
  - Add OpenAPI schemas for all DTOs
  - Include error response examples

## Observability

- [ ] **Enhanced logging**
  - Add request IDs (correlation IDs)
  - Log all database queries in dev
  - Add structured logging fields
  - Log cleanup duration metrics

- [ ] **Distributed tracing**
  - Add OpenTelemetry tracing
  - Trace database queries
  - Trace Redis operations
  - Trace gRPC calls to Tasks service
  - Export to Jaeger/Tempo

- [ ] **Metrics endpoint**
  - Add `/metrics` endpoint for Prometheus
  - Expose HTTP request metrics
  - Expose database pool metrics
  - Expose custom business metrics

## Security

- [ ] **Add authentication/authorization**
  - Implement JWT authentication
  - Add role-based access control (RBAC)
  - Protect all non-public endpoints
  - Add API key support for service-to-service

- [ ] **Enable CORS middleware**
  - Use `axum-helpers` CORS configuration
  - Set allowed origins from config
  - Configure CORS for production
  - Location: Uncomment in router setup

- [ ] **Add rate limiting**
  - Implement per-IP rate limits
  - Add per-user rate limits
  - Configure different limits per endpoint
  - Use Redis for distributed rate limiting

- [ ] **Security headers**
  - Already implemented via `axum-helpers`
  - ✅ X-Content-Type-Options: nosniff
  - ✅ X-Frame-Options: DENY
  - ✅ X-XSS-Protection
  - ✅ Referrer-Policy
  - ✅ Permissions-Policy

## Error Handling

- [ ] **Migrate to axum-helpers error types**
  - Use `AppError` enum from `axum-helpers`
  - Replace `ErrorResponse` with standard types
  - Consistent error responses across API
  - Location: `src/main.rs:49-51`, handler functions

- [ ] **Add error tracking**
  - Integrate Sentry or similar
  - Track error rates by endpoint
  - Alert on error spikes
  - Add error context (user, request ID, etc.)

## Testing

- [ ] **Add integration tests**
  - Test all HTTP endpoints
  - Test database operations
  - Test Redis operations
  - Test graceful shutdown
  - Location: Create `tests/integration_test.rs`

- [ ] **Add load tests**
  - Performance baselines
  - Concurrent user tests
  - Database query performance
  - Shutdown under load
  - Location: Create `tests/load_test.rs` or separate directory

- [ ] **Add contract tests**
  - Test gRPC contracts with Tasks service
  - Test API contracts for clients
  - Document breaking changes
  - Location: Create `tests/contract_test.rs`

## Performance

- [ ] **Database query optimization**
  - Add query logging in development
  - Identify slow queries
  - Add proper indexes
  - Consider read replicas

- [ ] **Response caching**
  - Cache frequently accessed data
  - Use Redis for cache layer
  - Implement cache invalidation strategy
  - Add cache-control headers

- [ ] **Connection pooling tuning**
  - Profile connection usage
  - Adjust pool sizes based on load
  - Monitor pool exhaustion
  - Consider connection pool per service

## Code Quality

- [ ] **Add more comprehensive logging**
  - Log all external service calls
  - Log authentication attempts
  - Log slow operations (> 1s)
  - Sanitize sensitive data from logs

- [ ] **Refactor handlers**
  - Extract business logic to services
  - Reduce handler complexity
  - Add handler tests
  - Location: `src/main.rs:110-239`

- [ ] **Configuration improvements**
  - Validate all config on startup
  - Add config documentation
  - Support multiple environments
  - Add config reload without restart

- [ ] **Error recovery**
  - Add retry logic for external services
  - Circuit breaker for Tasks gRPC service
  - Graceful degradation when Redis unavailable
  - Better error messages

## Infrastructure

- [ ] **Horizontal Pod Autoscaling (HPA)**
  - Configure HPA based on CPU/memory
  - Configure HPA based on custom metrics
  - Test scaling behavior
  - Document scaling strategy

- [ ] **Resource limits**
  - Set appropriate CPU/memory requests
  - Set CPU/memory limits
  - Test under resource constraints
  - Monitor OOM kills

- [ ] **Multi-region deployment**
  - Design for multi-region
  - Handle latency between regions
  - Database replication strategy
  - Session affinity considerations

## Documentation

- [ ] **Architecture documentation**
  - Document system architecture
  - Create sequence diagrams
  - Document data flow
  - Location: Create `docs/architecture.md`

- [ ] **Deployment guide**
  - Step-by-step deployment instructions
  - Environment setup guide
  - Rollback procedures
  - Location: Create `docs/deployment.md`

- [ ] **Troubleshooting guide**
  - Common issues and solutions
  - Log analysis guide
  - Performance debugging
  - Location: Create `docs/troubleshooting.md`

- [ ] **Development setup guide**
  - Local development instructions
  - Database setup
  - Testing guide
  - Location: Create `docs/development.md`

## Future Features

- [ ] **GraphQL API**
  - Add GraphQL endpoint alongside REST
  - Use async-graphql
  - Schema design
  - Migration strategy

- [ ] **WebSocket support**
  - Real-time updates
  - Connection management
  - Authentication for WebSockets
  - Graceful shutdown for WebSocket connections

- [ ] **Background job processing**
  - Integrate job queue (Redis-based or separate)
  - Schedule periodic tasks
  - Retry failed jobs
  - Monitor job execution

- [ ] **Audit logging**
  - Log all data modifications
  - Track who changed what and when
  - Compliance requirements
  - Audit log retention

## Completed ✅

- [x] **Production-ready shutdown**
  - Graceful shutdown with timeout
  - Database connection cleanup
  - Redis connection cleanup
  - Comprehensive logging
  - Documentation and testing guide

---

## Priority Legend

- **High Priority**: Should be done before production deployment
- **Medium Priority**: Improves production readiness and operations
- **Low Priority**: Nice to have, improves developer experience

## Notes

- Review and update this TODO list regularly
- Mark items as complete when done
- Add new items as they're identified
- Link to relevant issues/PRs when work starts
