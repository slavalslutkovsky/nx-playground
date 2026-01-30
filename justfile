#!/usr/bin/env just --justfile

default:
    just -l

# Full quality check for Rust monorepo (read-only, CI-safe)
check: fmt-check lint test audit
    @echo "All checks passed!"

# Check formatting without modifying files
fmt-check:
    cargo fmt --all --check

# Format all Rust code
fmt:
    cargo fmt --all

# Run clippy linter on all packages
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Run all tests
test:
    cargo nextest run --workspace

# Security and dependency checks
audit:
    cargo audit --ignore RUSTSEC-2023-0071  # RSA timing vulnerability - no fix available
    cargo deny check --config .cargo/deny.toml

# Quick check (no tests, just compile and lint)
check-quick: fmt-check
    cargo check --workspace
    cargo clippy --workspace --all-targets -- -D warnings

# Show outdated dependencies
outdated:
    cargo outdated --workspace

# Update Cargo.lock to latest compatible versions
update:
    cargo update

# Upgrade Cargo.toml versions to latest (requires cargo-edit)
upgrade:
    cargo upgrade --workspace --incompatible
    cargo update

_docker-up:
    docker compose -f manifests/dockers/compose.yaml up -d

# Remove local env db
docker-down:
    docker compose -f manifests/dockers/compose.yaml down

run *args:
    bacon {{ args }}

# Run zerg web dev server
web:
    cd apps/zerg/web && bun run dev

# Migrations now handled by SeaORM (libs/migration)
# Run via: cargo run --bin migration up

# Or automatically on app start with RUN_MIGRATIONS=true
_migration:
    @echo "Using SeaORM migrations. Run: cargo run --bin migration up"
    cargo run --bin migration up

# TODO: Create seed data migration if needed
_seed:
    @echo "Seed data not yet implemented for new schema"

sort-deps:
    cargo fmt
    cargo sort --workspace

#  cargo doc --workspace --no-deps --document-private-items --open
#  bacon doc --open

reset-db:
    just docker-down
    docker volume prune -af
    just _docker-up
    just _migration
    just _seed

schema:
    cargo run --bin schema-gen -- --format all -o docs

# docker rm $(docker ps -aq) -f
test-all:
    cargo nextest run --workspace

kompose:
    kubectl create ns dbs
    kompose convert --file ~/private/nx-playground/manifests/dockers/compose.yaml --namespace dbs --stdout | kubectl apply -f -
    just migrate-cluster

# Run migrations against Kind cluster postgres (port 5433)
migrate-cluster:
    DATABASE_URL="postgres://myuser:mypassword@localhost:5433/mydatabase" cargo run -p migration -- up

# Run migrations against local Docker postgres (port 5432)
migrate-local:
    DATABASE_URL="postgres://myuser:mypassword@localhost:5432/mydatabase" cargo run -p migration -- up

# Check migration status on cluster
migrate-status:
    DATABASE_URL="postgres://myuser:mypassword@localhost:5433/mydatabase" cargo run -p migration -- status

# Proto/gRPC workflow (using buf)
# Directory containing buf configuration

proto_dir := "manifests/grpc"

# Format proto files
proto-fmt:
    cd {{ proto_dir }} && buf format -w

# Lint proto files
proto-lint:
    cd {{ proto_dir }} && buf lint

# Check for breaking changes (against git main branch)
proto-breaking:
    cd {{ proto_dir }} && buf breaking --against '.git#branch=main'

# Build/validate proto files
proto-build:
    cd {{ proto_dir }} && buf build

# Generate Rust code from proto files
proto-gen:
    cd {{ proto_dir }} && buf generate

# Verify generated Rust code compiles
proto-check:
    cargo check -p rpc

# Full proto workflow: format, lint, build, generate, verify
proto: proto-fmt proto-lint proto-build proto-gen proto-check
    @echo "Proto workflow complete"

# Alias for backward compatibility
buf: proto

# Benchmark tasks API endpoints with wrk
# Directory containing wrk scripts

wrk_dir := "scripts/wrk"
api_url_local := "http://localhost:8080/api"
api_url_cluster := "http://localhost:5221/api"

# ============================================================================
# Local Benchmarks (localhost:8080)
# ============================================================================

# Benchmark GET /api/tasks (gRPC endpoint) - Local
bench-tasks-grpc:
    @echo "=== Benchmarking gRPC Tasks Endpoint (GET) - Local ==="
    wrk -t4 -c50 -d30s --latency -s {{ wrk_dir }}/report.lua {{ api_url_local }}/tasks

# Benchmark GET /api/tasks-direct (Direct DB endpoint) - Local
bench-tasks-direct:
    @echo "=== Benchmarking Direct DB Tasks Endpoint (GET) - Local ==="
    wrk -t4 -c50 -d30s --latency -s {{ wrk_dir }}/report.lua {{ api_url_local }}/tasks-direct

# Benchmark POST /api/tasks (gRPC endpoint) - Local
bench-tasks-grpc-post:
    @echo "=== Benchmarking gRPC Tasks Endpoint (POST) - Local ==="
    wrk -t4 -c50 -d30s --latency -s {{ wrk_dir }}/post-task.lua {{ api_url_local }}/tasks

# Benchmark POST /api/tasks-direct (Direct DB endpoint) - Local
bench-tasks-direct-post:
    @echo "=== Benchmarking Direct DB Tasks Endpoint (POST) - Local ==="
    wrk -t4 -c50 -d30s --latency -s {{ wrk_dir }}/post-task.lua {{ api_url_local }}/tasks-direct

# Run all local benchmarks and compare
bench-tasks-compare:
    @echo "======================================"
    @echo "  Tasks API Benchmark Comparison (Local)"
    @echo "======================================"
    @echo ""
    just bench-tasks-grpc
    @echo ""
    just bench-tasks-direct
    @echo ""
    @echo "======================================"
    @echo "  POST Benchmarks"
    @echo "======================================"
    @echo ""
    just bench-tasks-grpc-post
    @echo ""
    just bench-tasks-direct-post

# ============================================================================
# Cluster Benchmarks (Kind via Tilt port-forward on localhost:5221)
# ============================================================================

# Benchmark GET /api/tasks (gRPC endpoint) - Cluster
bench-cluster-tasks-grpc:
    @echo "=== Benchmarking gRPC Tasks Endpoint (GET) - Cluster ==="
    wrk -t4 -c50 -d30s --latency -s {{ wrk_dir }}/report.lua {{ api_url_cluster }}/tasks

# Benchmark GET /api/tasks-direct (Direct DB endpoint) - Cluster
bench-cluster-tasks-direct:
    @echo "=== Benchmarking Direct DB Tasks Endpoint (GET) - Cluster ==="
    wrk -t4 -c50 -d30s --latency -s {{ wrk_dir }}/report.lua {{ api_url_cluster }}/tasks-direct

# Benchmark POST /api/tasks (gRPC endpoint) - Cluster
bench-cluster-tasks-grpc-post:
    @echo "=== Benchmarking gRPC Tasks Endpoint (POST) - Cluster ==="
    wrk -t4 -c50 -d30s --latency -s {{ wrk_dir }}/post-task.lua {{ api_url_cluster }}/tasks

# Benchmark POST /api/tasks-direct (Direct DB endpoint) - Cluster
bench-cluster-tasks-direct-post:
    @echo "=== Benchmarking Direct DB Tasks Endpoint (POST) - Cluster ==="
    wrk -t4 -c50 -d30s --latency -s {{ wrk_dir }}/post-task.lua {{ api_url_cluster }}/tasks-direct

# Run all cluster benchmarks and compare
bench-cluster-compare:
    @echo "======================================"
    @echo "  Tasks API Benchmark Comparison (Cluster)"
    @echo "======================================"
    @echo ""
    just bench-cluster-tasks-grpc
    @echo ""
    just bench-cluster-tasks-direct
    @echo ""
    @echo "======================================"
    @echo "  POST Benchmarks (Cluster)"
    @echo "======================================"
    @echo ""
    just bench-cluster-tasks-grpc-post
    @echo ""
    just bench-cluster-tasks-direct-post

# Quick cluster benchmark (10s duration, lighter load)
bench-cluster-quick:
    @echo "=== Quick Benchmark: gRPC GET (Cluster) ==="
    wrk -t2 -c10 -d10s --latency {{ api_url_cluster }}/tasks
    @echo ""
    @echo "=== Quick Benchmark: Direct DB GET (Cluster) ==="
    wrk -t2 -c10 -d10s --latency {{ api_url_cluster }}/tasks-direct
    @echo ""
    @echo "Benchmark complete!"

# Quick benchmark (10s duration, lighter load) - Local
bench-tasks-quick:
    @echo "=== Quick Benchmark: gRPC GET (Local) ==="
    wrk -t2 -c10 -d10s --latency {{ api_url_local }}/tasks
    @echo ""
    @echo "=== Quick Benchmark: Direct DB GET (Local) ==="
    wrk -t2 -c10 -d10s --latency {{ api_url_local }}/tasks-direct

backstage-dev:
    kubectl apply -k manifests/kustomize/backstage/overlays/dev

backstage-prod:
    kubectl apply -k manifests/kustomize/backstage/overlays/prod

backstage-logs:
    kubectl logs -n backstage deployment/backstage -f

backstage-catalog-generate:
    nu scripts/nu/generate-backstage-catalog.nu

crossplane-functions-install:
    echo 'apiVersion: pkg.crossplane.io/v1beta1\nkind: Function\nmetadata:\n  name: function-kcl\nspec:\n  package: docker.io/kcllang/function-kcl:latest' | kubectl apply -f -
    echo 'apiVersion: pkg.crossplane.io/v1beta1\nkind: Function\nmetadata:\n  name: function-cue\nspec:\n  package: docker.io/crossplane-contrib/function-cue:latest' | kubectl apply -f -

backstage-setup-github:
    nu scripts/nu/backstage-setup-providers.nu github

backstage-setup-aws:
    nu scripts/nu/backstage-setup-providers.nu aws

backstage-setup-gcp:
    nu scripts/nu/backstage-setup-providers.nu gcp

backstage-setup-cloudflare:
    nu scripts/nu/backstage-setup-providers.nu cloudflare

backstage-setup-all:
    nu scripts/nu/backstage-setup-providers.nu all

backstage-restart:
    kubectl rollout restart deployment/backstage -n backstage
    kubectl rollout status deployment/backstage -n backstage

# ============================================================================
# Local Development Environment
# ============================================================================

# Start full local dev environment (Kind + DBs + Secrets + Tilt)
local-up *args:
    nu scripts/nu/mod.nu up {{args}}

# Tear down local dev environment
local-down *args:
    nu scripts/nu/mod.nu down {{args}}

# Quick restart (keep cluster, redeploy apps)
local-restart:
    nu scripts/nu/mod.nu down --keep-cluster
    tilt up

# Show environment status
local-status:
    nu scripts/nu/mod.nu status
