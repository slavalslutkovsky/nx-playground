# Dagger CI/CD Module

Dagger pipelines for building, testing, and deploying NX Playground.

## Prerequisites

Install Dagger CLI:
```bash
# macOS
brew install dagger/tap/dagger

# Linux
curl -fsSL https://dl.dagger.io/dagger/install.sh | sh
```

## Setup

```bash
cd dagger
npm install
dagger develop
```

## Available Functions

### Database Services

Start individual databases for development or testing:

```bash
# Qdrant (vector database)
dagger call qdrant up

# Neo4j (graph database)
dagger call neo4j up

# ArangoDB (multi-model)
dagger call arangodb up

# Milvus (vector database)
dagger call milvus up

# PostgreSQL
dagger call postgres up

# Redis
dagger call redis up
```

### Build & Test

```bash
# Check compilation
dagger call check --source=..

# Run linter (clippy)
dagger call lint --source=..

# Check formatting
dagger call format-check --source=..

# Run tests
dagger call test --source=..

# Run tests with databases
dagger call test-with-databases --source=..

# Build release
dagger call build --source=..
```

### Container Builds

```bash
# Build zerg-api container
dagger call build-zerg-api --source=..

# Build zerg-tasks container
dagger call build-zerg-tasks --source=..

# Build and export as tarball
dagger call build-zerg-api --source=.. export --path=./zerg-api.tar
```

### CI Pipeline

```bash
# Run full CI (check, lint, format, test)
dagger call ci --source=..

# Run CI/CD (CI + build containers)
dagger call cicd --source=..

# CI/CD with push to registry
dagger call cicd --source=.. \
  --registry=ghcr.io \
  --repository=yurikrupnik/nx-playground \
  --tag=latest \
  --push=true
```

### Development Environment

```bash
# Start all databases for development
dagger call dev-env --source=..

# Get environment with all databases
dagger call all-databases
```

## Function Reference

| Function | Description |
|----------|-------------|
| `qdrant()` | Qdrant vector database service |
| `neo4j(password)` | Neo4j graph database service |
| `arangodb(password)` | ArangoDB multi-model database |
| `milvus()` | Milvus vector database (with etcd + minio) |
| `postgres(db, user, pass)` | PostgreSQL database |
| `redis()` | Redis cache |
| `allDatabases()` | Container with all DB services bound |
| `rustBuilder()` | Rust build container with caching |
| `build(source)` | Build all Rust packages |
| `check(source)` | Run cargo check |
| `lint(source)` | Run cargo clippy |
| `formatCheck(source)` | Run cargo fmt --check |
| `test(source)` | Run cargo test |
| `testWithDatabases(source)` | Run tests with PG + Redis |
| `buildZergApi(source)` | Build zerg-api container |
| `buildZergTasks(source)` | Build zerg-tasks container |
| `publish(container, registry, repo, tag)` | Push container to registry |
| `ci(source)` | Full CI pipeline |
| `cicd(source, registry, repo, tag, push)` | Full CI/CD pipeline |
| `devEnv(source)` | Development environment |

## GitHub Actions Integration

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  ci:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dagger/dagger-for-github@v6
        with:
          version: "latest"
      - name: Run CI
        run: dagger call ci --source=.

  build:
    runs-on: ubuntu-latest
    needs: ci
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4
      - uses: dagger/dagger-for-github@v6
        with:
          version: "latest"
      - name: Build and Push
        run: |
          dagger call cicd --source=. \
            --registry=ghcr.io \
            --repository=${{ github.repository }} \
            --tag=${{ github.sha }} \
            --push=true
        env:
          DAGGER_CLOUD_TOKEN: ${{ secrets.DAGGER_CLOUD_TOKEN }}
```

## Local Development with Databases

```bash
# Terminal 1: Start databases
cd dagger
dagger call all-databases

# Terminal 2: Run your app with env vars
export QDRANT_URL=http://localhost:6333
export NEO4J_URI=bolt://localhost:7687
export NEO4J_USER=neo4j
export NEO4J_PASSWORD=password123
cargo run -p zerg_api
```

## Caching

The module uses Dagger cache volumes for:
- Cargo registry (`cargo-cache`)
- Rust target directory (`rust-target-cache`)

This significantly speeds up subsequent builds.
