#!/usr/bin/env just --justfile

default:
  just -l

check:
  cargo check
  cargo audit

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
  kompose convert --file /Users/yurikrupnik/projects/playground/manifests/dockers/compose.yaml -o k8s-manifests/
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

# Benchmark tasks API endpoints with wrk
# Directory containing wrk scripts
wrk_dir := "scripts/wrk"
api_url := "http://localhost:8080/api"

# Benchmark GET /api/tasks (gRPC endpoint)
bench-tasks-grpc:
    @echo "=== Benchmarking gRPC Tasks Endpoint (GET) ==="
    wrk -t4 -c50 -d30s --latency -s {{wrk_dir}}/report.lua {{api_url}}/tasks

# Benchmark GET /api/tasks-direct (Direct DB endpoint)
bench-tasks-direct:
    @echo "=== Benchmarking Direct DB Tasks Endpoint (GET) ==="
    wrk -t4 -c50 -d30s --latency -s {{wrk_dir}}/report.lua {{api_url}}/tasks-direct

# Benchmark POST /api/tasks (gRPC endpoint)
bench-tasks-grpc-post:
    @echo "=== Benchmarking gRPC Tasks Endpoint (POST) ==="
    wrk -t4 -c50 -d30s --latency -s {{wrk_dir}}/post-task.lua {{api_url}}/tasks

# Benchmark POST /api/tasks-direct (Direct DB endpoint)
bench-tasks-direct-post:
    @echo "=== Benchmarking Direct DB Tasks Endpoint (POST) ==="
    wrk -t4 -c50 -d30s --latency -s {{wrk_dir}}/post-task.lua {{api_url}}/tasks-direct

# Run all benchmarks and compare
bench-tasks-compare:
    @echo "======================================"
    @echo "  Tasks API Benchmark Comparison"
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
    @echo ""
    @echo "Benchmark complete!"

# Quick benchmark (10s duration, lighter load)
bench-tasks-quick:
    @echo "=== Quick Benchmark: gRPC GET ==="
    wrk -t2 -c10 -d10s --latency {{api_url}}/tasks
    @echo ""
    @echo "=== Quick Benchmark: Direct DB GET ==="
    wrk -t2 -c10 -d10s --latency {{api_url}}/tasks-direct

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
