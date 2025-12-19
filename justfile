#!/usr/bin/env just --justfile

default:
  just -l

check:
  cargo check
  cargo audit

# Sync AI coding assistant rules from CLAUDE.md to other tool formats
sync-ai-rules:
  @echo "Syncing AI rules from CLAUDE.md..."
  @# Extract content after the header for other tools
  @tail -n +4 CLAUDE.md > /tmp/ai-rules-content.md
  @echo "# Cursor Rules" > .cursorrules
  @echo "# This file is synced from CLAUDE.md - edit CLAUDE.md as the source of truth" >> .cursorrules
  @echo "" >> .cursorrules
  @cat /tmp/ai-rules-content.md >> .cursorrules
  @echo "# GitHub Copilot Instructions" > .github/copilot-instructions.md
  @echo "<!-- This file is synced from CLAUDE.md - edit CLAUDE.md as the source of truth -->" >> .github/copilot-instructions.md
  @echo "" >> .github/copilot-instructions.md
  @cat /tmp/ai-rules-content.md >> .github/copilot-instructions.md
  @echo "# Windsurf Rules" > .windsurfrules
  @echo "# This file is synced from CLAUDE.md - edit CLAUDE.md as the source of truth" >> .windsurfrules
  @echo "" >> .windsurfrules
  @cat /tmp/ai-rules-content.md >> .windsurfrules
  @rm /tmp/ai-rules-content.md
  @echo "Done! Synced to: .cursorrules, .github/copilot-instructions.md, .windsurfrules"

_docker-up:
  docker compose -f manifests/dockers/compose.yaml up -d
# Remove local env db
docker-down:
  docker compose -f manifests/dockers/compose.yaml down

run *args:
  bacon {{args}}

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
#  docker rm $(docker ps -aq) -f
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
    cd {{proto_dir}} && buf format -w

# Lint proto files
proto-lint:
    cd {{proto_dir}} && buf lint

# Check for breaking changes (against git main branch)
proto-breaking:
    cd {{proto_dir}} && buf breaking --against '.git#branch=main'

# Build/validate proto files
proto-build:
    cd {{proto_dir}} && buf build

# Generate Rust code from proto files
proto-gen:
    cd {{proto_dir}} && buf generate

# Verify generated Rust code compiles
proto-check:
    cargo check -p rpc

# Full proto workflow: format, lint, build, generate, verify
proto: proto-fmt proto-lint proto-build proto-gen proto-check
    @echo "Proto workflow complete"

# Alias for backward compatibility
buf: proto

# ============================================================================
# Cloud Cost Optimization Protos (libs/protos)
# ============================================================================

# Format cloud proto files
proto-cloud-fmt:
    cd libs/protos && buf format -w

# Lint cloud proto files
proto-cloud-lint:
    cd libs/protos && buf lint

# Build/validate cloud proto files
proto-cloud-build:
    cd libs/protos && buf build

# Generate Rust code from cloud proto files
proto-cloud-gen:
    cd libs/protos && buf generate

# Verify generated cloud proto Rust code compiles
proto-cloud-check:
    cargo check -p protos

# Full cloud proto workflow: format, lint, build, generate, verify
proto-cloud: proto-cloud-fmt proto-cloud-lint proto-cloud-build proto-cloud-gen proto-cloud-check
    @echo "Cloud proto workflow complete"

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
    wrk -t4 -c50 -d30s --latency -s {{wrk_dir}}/report.lua {{api_url_local}}/tasks

# Benchmark GET /api/tasks-direct (Direct DB endpoint) - Local
bench-tasks-direct:
    @echo "=== Benchmarking Direct DB Tasks Endpoint (GET) - Local ==="
    wrk -t4 -c50 -d30s --latency -s {{wrk_dir}}/report.lua {{api_url_local}}/tasks-direct

# Benchmark POST /api/tasks (gRPC endpoint) - Local
bench-tasks-grpc-post:
    @echo "=== Benchmarking gRPC Tasks Endpoint (POST) - Local ==="
    wrk -t4 -c50 -d30s --latency -s {{wrk_dir}}/post-task.lua {{api_url_local}}/tasks

# Benchmark POST /api/tasks-direct (Direct DB endpoint) - Local
bench-tasks-direct-post:
    @echo "=== Benchmarking Direct DB Tasks Endpoint (POST) - Local ==="
    wrk -t4 -c50 -d30s --latency -s {{wrk_dir}}/post-task.lua {{api_url_local}}/tasks-direct

# Benchmark GET /api/tasks-stream (Stream async - fire-and-forget) - Local
bench-tasks-stream-async:
    @echo "=== Benchmarking Stream Async Tasks Endpoint (GET) - Local ==="
    wrk -t4 -c50 -d30s --latency -s {{wrk_dir}}/report.lua {{api_url_local}}/tasks-stream

# Benchmark POST /api/tasks-stream (Stream async - fire-and-forget) - Local
bench-tasks-stream-async-post:
    @echo "=== Benchmarking Stream Async Tasks Endpoint (POST) - Local ==="
    wrk -t4 -c50 -d30s --latency -s {{wrk_dir}}/post-task.lua {{api_url_local}}/tasks-stream

# Run all local benchmarks and compare (GET only)
bench-tasks-compare:
    @echo "======================================"
    @echo "  Tasks API Benchmark Comparison (Local)"
    @echo "  GET Requests - 3 Approaches"
    @echo "======================================"
    @echo ""
    @echo "--- 1. gRPC (via zerg-tasks service) ---"
    just bench-tasks-grpc
    @echo ""
    @echo "--- 2. Direct DB (direct PostgreSQL) ---"
    just bench-tasks-direct
    @echo ""
    @echo "--- 3. Stream (fire-and-forget, 202 Accepted) ---"
    just bench-tasks-stream-async

# Run all local POST benchmarks
bench-tasks-compare-post:
    @echo "======================================"
    @echo "  Tasks API Benchmark Comparison (Local)"
    @echo "  POST Requests - 3 Approaches"
    @echo "======================================"
    @echo ""
    @echo "--- 1. gRPC (via zerg-tasks service) ---"
    just bench-tasks-grpc-post
    @echo ""
    @echo "--- 2. Direct DB (direct PostgreSQL) ---"
    just bench-tasks-direct-post
    @echo ""
    @echo "--- 3. Stream (fire-and-forget, 202 Accepted) ---"
    just bench-tasks-stream-async-post

# Run full comparison (all 3 approaches, GET and POST)
bench-tasks-full:
    just bench-tasks-compare
    @echo ""
    just bench-tasks-compare-post

# ============================================================================
# Cluster Benchmarks (Kind via Tilt port-forward on localhost:5221)
# ============================================================================

# Benchmark GET /api/tasks (gRPC endpoint) - Cluster
bench-cluster-tasks-grpc:
    @echo "=== Benchmarking gRPC Tasks Endpoint (GET) - Cluster ==="
    wrk -t4 -c50 -d30s --latency -s {{wrk_dir}}/report.lua {{api_url_cluster}}/tasks

# Benchmark GET /api/tasks-direct (Direct DB endpoint) - Cluster
bench-cluster-tasks-direct:
    @echo "=== Benchmarking Direct DB Tasks Endpoint (GET) - Cluster ==="
    wrk -t4 -c50 -d30s --latency -s {{wrk_dir}}/report.lua {{api_url_cluster}}/tasks-direct

# Benchmark POST /api/tasks (gRPC endpoint) - Cluster
bench-cluster-tasks-grpc-post:
    @echo "=== Benchmarking gRPC Tasks Endpoint (POST) - Cluster ==="
    wrk -t4 -c50 -d30s --latency -s {{wrk_dir}}/post-task.lua {{api_url_cluster}}/tasks

# Benchmark POST /api/tasks-direct (Direct DB endpoint) - Cluster
bench-cluster-tasks-direct-post:
    @echo "=== Benchmarking Direct DB Tasks Endpoint (POST) - Cluster ==="
    wrk -t4 -c50 -d30s --latency -s {{wrk_dir}}/post-task.lua {{api_url_cluster}}/tasks-direct

# Benchmark GET /api/tasks-stream (Stream async) - Cluster
bench-cluster-tasks-stream-async:
    @echo "=== Benchmarking Stream Async Tasks Endpoint (GET) - Cluster ==="
    wrk -t4 -c50 -d30s --latency -s {{wrk_dir}}/report.lua {{api_url_cluster}}/tasks-stream

# Benchmark GET /api/tasks-stream-sync (Stream sync) - Cluster
bench-cluster-tasks-stream-sync:
    @echo "=== Benchmarking Stream Sync Tasks Endpoint (GET) - Cluster ==="
    wrk -t4 -c50 -d30s --latency -s {{wrk_dir}}/report.lua {{api_url_cluster}}/tasks-stream-sync

# Benchmark POST /api/tasks-stream (Stream async) - Cluster
bench-cluster-tasks-stream-async-post:
    @echo "=== Benchmarking Stream Async Tasks Endpoint (POST) - Cluster ==="
    wrk -t4 -c50 -d30s --latency -s {{wrk_dir}}/post-task.lua {{api_url_cluster}}/tasks-stream

# Benchmark POST /api/tasks-stream-sync (Stream sync) - Cluster
bench-cluster-tasks-stream-sync-post:
    @echo "=== Benchmarking Stream Sync Tasks Endpoint (POST) - Cluster ==="
    wrk -t4 -c50 -d30s --latency -s {{wrk_dir}}/post-task.lua {{api_url_cluster}}/tasks-stream-sync

# Run all cluster benchmarks and compare (GET)
bench-cluster-compare:
    @echo "======================================"
    @echo "  Tasks API Benchmark Comparison (Cluster)"
    @echo "  GET Requests - 4 Approaches"
    @echo "======================================"
    @echo ""
    @echo "--- 1. gRPC ---"
    just bench-cluster-tasks-grpc
    @echo ""
    @echo "--- 2. Direct DB ---"
    just bench-cluster-tasks-direct
    @echo ""
    @echo "--- 3. Stream Async (fire-and-forget) ---"
    just bench-cluster-tasks-stream-async
    @echo ""
    @echo "--- 4. Stream Sync (waits for result) ---"
    just bench-cluster-tasks-stream-sync

# Run all cluster POST benchmarks
bench-cluster-compare-post:
    @echo "======================================"
    @echo "  Tasks API Benchmark Comparison (Cluster)"
    @echo "  POST Requests - 4 Approaches"
    @echo "======================================"
    @echo ""
    @echo "--- 1. gRPC ---"
    just bench-cluster-tasks-grpc-post
    @echo ""
    @echo "--- 2. Direct DB ---"
    just bench-cluster-tasks-direct-post
    @echo ""
    @echo "--- 3. Stream Async (fire-and-forget) ---"
    just bench-cluster-tasks-stream-async-post
    @echo ""
    @echo "--- 4. Stream Sync (waits for result) ---"
    just bench-cluster-tasks-stream-sync-post

# Run full cluster comparison
bench-cluster-full:
    just bench-cluster-compare
    @echo ""
    just bench-cluster-compare-post

# Quick cluster benchmark (10s duration, lighter load)
bench-cluster-quick:
    @echo "=== Quick Benchmark: gRPC GET (Cluster) ==="
    wrk -t2 -c10 -d10s --latency {{api_url_cluster}}/tasks
    @echo ""
    @echo "=== Quick Benchmark: Direct DB GET (Cluster) ==="
    wrk -t2 -c10 -d10s --latency {{api_url_cluster}}/tasks-direct
    @echo ""
    @echo "=== Quick Benchmark: Stream Async GET (Cluster) ==="
    wrk -t2 -c10 -d10s --latency {{api_url_cluster}}/tasks-stream
    @echo ""
    @echo "=== Quick Benchmark: Stream Sync GET (Cluster) ==="
    wrk -t2 -c10 -d10s --latency {{api_url_cluster}}/tasks-stream-sync
    @echo ""
    @echo "Benchmark complete!"

# Quick benchmark (10s duration, lighter load) - Local
bench-tasks-quick:
    @echo "=== Quick Benchmark: gRPC GET (Local) ==="
    wrk -t2 -c10 -d10s --latency {{api_url_local}}/tasks
    @echo ""
    @echo "=== Quick Benchmark: Direct DB GET (Local) ==="
    wrk -t2 -c10 -d10s --latency {{api_url_local}}/tasks-direct
    @echo ""
    @echo "=== Quick Benchmark: Stream Async GET (Local) ==="
    wrk -t2 -c10 -d10s --latency {{api_url_local}}/tasks-stream
    @echo ""
    @echo "=== Quick Benchmark: Stream Sync GET (Local) ==="
    wrk -t2 -c10 -d10s --latency {{api_url_local}}/tasks-stream-sync
    @echo ""
    @echo "Benchmark complete!"

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

dsa:
  nx add @nx/vite
  nx g @nx/vite:app web --directory=apps/zerg/web --unitTestRunner=vitest --projectNameAndRootFormat=as-provided
  cargo run -- migrate up # test is why it is up like this
