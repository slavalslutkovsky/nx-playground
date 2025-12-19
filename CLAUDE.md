# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Nx monorepo with Rust backend (Axum, Tonic, SeaORM) and SolidJS frontend. Uses bun for JS package management and @monodon/rust for Nx+Rust integration.

## Build & Run Commands

```bash
# Prerequisites
brew install direnv just
eval "$(direnv hook zsh)"
cp .env.example .env && direnv allow

# Infrastructure (Docker Compose: PostgreSQL, Redis, MongoDB, Mailpit)
just _docker-up                    # Start services
just docker-down                   # Stop services
just reset-db                      # Fresh DB (down + volume prune + up + migrate)

# Migrations (SeaORM)
just migrate-local                 # Local Docker (port 5432)
just migrate-cluster               # Kind cluster (port 5433)
sea-orm-cli migrate -d libs/migration generate <name>  # Create new migration

# Run services
cargo run -p zerg_api              # REST API (port 8080)
cargo run -p zerg_tasks            # gRPC Tasks service (port 50051)
cargo run -p zerg_tasks_worker     # Redis Streams task consumer
cargo run -p zerg_email_worker     # Redis Streams email consumer
just web                           # SolidJS frontend (port 3000)

# Quality checks
cargo check                        # Rust compilation check
cargo clippy --workspace           # Rust linting
cargo fmt                          # Rust formatting
bun nx affected -t lint            # Nx affected lint
bun nx affected -t test            # Nx affected test
bun nx affected -t build           # Nx affected build

# Testing
cargo test --workspace             # All tests
cargo test -p domain_tasks --lib   # Unit tests for a crate
cargo test -p domain_tasks --test integration_test  # Integration tests
cargo nextest run --workspace      # Faster test runner

# gRPC (Protocol Buffers in manifests/grpc/)
just proto                         # Full workflow: fmt, lint, build, generate
just proto-gen                     # Generate Rust code from .proto files
```

## Architecture

### Applications (apps/zerg/)

| App | Port | Purpose |
|-----|------|---------|
| api | 8080 | REST gateway, aggregates domains, gRPC client to tasks |
| tasks | 50051 | gRPC service for task CRUD |
| tasks-worker | - | Redis Streams consumer for async task processing |
| email-worker | - | Redis Streams consumer for email jobs |
| web | 3000 | SolidJS frontend |

### Libraries

**Core (libs/core/):**
- `axum-helpers` - JWT auth, Redis sessions, server setup, middleware
- `config` - `FromEnv` trait for environment-based configuration
- `grpc` - gRPC client pooling and connection management
- `stream-worker` - Generic Redis Streams consumer with DLQ, retries, metrics
- `proc_macros` - Derive macros: `ApiResource`, `SeaOrmResource`, `SelectableFields`
- `field-selector` - GraphQL-like `?fields=id,name` query parameter

**Domains (libs/domains/):**
Each domain follows 4-layer architecture:
```
handlers.rs  → HTTP routes (Axum Router)
service.rs   → Business logic, validation
repository.rs + postgres.rs → Data access (trait + PostgreSQL impl)
models.rs    → Entities, DTOs
error.rs     → Domain-specific errors
```

Domains: `tasks`, `projects`, `users`, `notifications`, `cloud_resources`

**Other:**
- `libs/rpc` - Generated gRPC code (from `buf generate`)
- `libs/migration` - SeaORM database migrations
- `libs/database` - PostgreSQL and Redis connection management

### Communication Patterns

1. **Direct DB** - API → PostgreSQL (lowest latency)
2. **gRPC** - API → zerg-tasks → PostgreSQL (service isolation)
3. **Redis Streams** - API → Stream → Worker → PostgreSQL (async, fire-and-forget)

## Key Environment Variables

```bash
DATABASE_URL=postgres://myuser:mypassword@localhost/mydatabase
REDIS_HOST=redis://localhost:6379
TASKS_SERVICE_ADDR=http://[::1]:50051
JWT_SECRET=dev-secret
PORT=8080
RUN_MIGRATIONS=true  # Auto-run migrations on API startup
```

## Kubernetes Development

```bash
kind create cluster --config manifests/kind-config.yaml
tilt up                            # Deploy with Tilt, access via localhost:5221
kubectl apply -k manifests/kustomize/dev  # Manual deploy
```

## CI/CD

GitHub Actions (`.github/workflows/ci-optimized.yml`):
- Uses sccache with GCS backend for Rust compilation caching
- Runs `bun nx affected -t lint/test/build`
- Requires `NX_CLOUD_ACCESS_TOKEN` and `GCP_SA_KEY` secrets
