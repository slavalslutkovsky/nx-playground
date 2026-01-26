#!/usr/bin/env just --justfile

default:
    just -l

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

# ============================================================================
# Quality Checks
# ============================================================================

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

# ============================================================================
# Dependency Updates
# ============================================================================

# Show outdated dependencies (cargo + bun)
outdated:
    @echo "=== Cargo Outdated ==="
    cargo outdated --workspace || true
    @echo ""
    @echo "=== Bun Outdated ==="
    bun outdated || true

# Update all lockfiles (Cargo.lock + bun.lock)
update:
    cargo update
    bun update

# Upgrade all dependencies to latest versions
upgrade:
    cargo upgrade --incompatible
    cargo update
    bun update --latest

# Update Cargo only
cargo-update:
    cargo update

# Upgrade Cargo.toml to latest versions (requires cargo-edit)
cargo-upgrade:
    cargo upgrade --workspace --incompatible
    cargo update

# Update bun packages only
bun-update:
    bun update

# Upgrade bun packages to latest (ignores semver)
bun-upgrade:
    bun update --latest

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
#    biome check . --write

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

dsa:
    nx add @nx/vite
    nx g @nx/vite:app web --directory=apps/zerg/web --unitTestRunner=vitest --projectNameAndRootFormat=as-provided
    cargo run -- migrate up # test is why it is up like this

# ============================================================================
# Environment Lifecycle (nu scripts)
# ============================================================================

# Bring up full dev environment (Kind cluster + services)
up *args:
    nu scripts/nu/mod.nu up {{args}}

# Tear down dev environment
down *args:
    nu scripts/nu/mod.nu down {{args}}

# Show environment status
status:
    nu scripts/nu/mod.nu status

# ============================================================================
# Crossplane
# ============================================================================

# Apply Crossplane functions (function-kcl, function-auto-ready)
crossplane-functions:
    kubectl apply -f manifests/crossplane/base/functions.yaml

# Install AWS providers
crossplane-providers-aws:
    kubectl apply -f manifests/crossplane/base/providers/aws.yaml

# Install GCP providers
crossplane-providers-gcp:
    kubectl apply -f manifests/crossplane/base/providers/gcp.yaml

# Install all providers (AWS + GCP)
crossplane-providers:
    kubectl apply -k manifests/crossplane/base/providers/

# Configure AWS credentials - use: kubectl apply -f manifests/crossplane/base/providerconfigs/aws.yaml
crossplane-config-aws:
    @echo "Configure AWS: kubectl apply -f manifests/crossplane/base/providerconfigs/aws.yaml"

# Configure GCP credentials - use: kubectl apply -f manifests/crossplane/base/providerconfigs/gcp.yaml
crossplane-config-gcp:
    @echo "Configure GCP: kubectl apply -f manifests/crossplane/base/providerconfigs/gcp.yaml"

# Show provider status
crossplane-providers-status:
    kubectl get providers

# Apply all Crossplane XRDs and Compositions (dev)
crossplane-apply-dev:
    kubectl apply -k manifests/crossplane/overlays/dev

# Apply all Crossplane XRDs and Compositions (prod)
crossplane-apply-prod:
    kubectl apply -k manifests/crossplane/overlays/prod

# Apply all Crossplane base resources
crossplane-apply:
    kubectl apply -k manifests/crossplane/base

# Delete all Crossplane resources
crossplane-delete:
    kubectl delete -k manifests/crossplane/base --ignore-not-found

# Show Crossplane composite resources
crossplane-composites:
    kubectl get composite

# Show Crossplane managed resources
crossplane-managed:
    kubectl get managed

# Show Crossplane claims
crossplane-claims:
    kubectl get claim --all-namespaces

# Full Crossplane setup (functions + providers + XRDs + compositions)
crossplane-setup:
    just crossplane-functions
    just crossplane-providers
    @echo "Waiting for functions and providers to install..."
    sleep 30
    just crossplane-apply
    @echo "Crossplane setup complete (configure credentials with crossplane-config-aws/gcp)"

# Watch Crossplane resources
crossplane-watch:
    watch -n 2 'kubectl get xrd,composition,composite,managed 2>/dev/null | head -50'
kube-get:
  kubectl get bucket.storage.gcp.upbound.io my-app-data -o yaml

# ============================================================================
# Platform (Upbound CLI) - KCL Schema-Driven Crossplane
# ============================================================================

platform_dir := "platform"
schema_registry := "docker.io/yurikrupnik/platform-schemas"

# ============================================================================
# Schema Management (OCI Registry)
# ============================================================================

# Login to Docker Hub for KCL registry
schema-login:
    kcl registry login docker.io

# Publish schemas to OCI registry (uses version from schemas/kcl.mod)
# Note: KCL uses the package version from kcl.mod, not the URL tag
schema-publish:
    cd {{ platform_dir }}/schemas && kcl mod push oci://{{ schema_registry }} --force
    @echo "Published schemas (version from kcl.mod)"

# Bump schema version and publish
schema-publish-version version:
    cd {{ platform_dir }}/schemas && sed -i '' 's|version = ".*"|version = "{{ version }}"|g' kcl.mod
    cd {{ platform_dir }}/schemas && kcl mod push oci://{{ schema_registry }} --force
    @echo "Published {{ schema_registry }}:{{ version }}"

# Update all function kcl.mod files to use a specific schema version
platform-update-schema-refs version:
    #!/usr/bin/env bash
    for mod in {{ platform_dir }}/functions/*/kcl.mod; do
        sed -i '' 's|tag = ".*"|tag = "{{ version }}"|g' "$mod"
    done
    echo "Updated all functions to use schemas:{{ version }}"

# ============================================================================
# XRD Generation
# ============================================================================

# Generate all XRD definitions from KCL schemas
platform-gen-xrds:
    cd {{ platform_dir }} && kcl run render/bucket_xrd.k > apis/xbuckets/definition.yaml
    cd {{ platform_dir }} && kcl run render/database_xrd.k > apis/xdatabases/definition.yaml
    cd {{ platform_dir }} && kcl run render/registry_xrd.k > apis/xregistries/definition.yaml
    cd {{ platform_dir }} && kcl run render/application_xrd.k > apis/xapplications/definition.yaml
    cd {{ platform_dir }} && kcl run render/network_xrd.k > apis/xnetworks/definition.yaml
    @echo "All XRD definitions generated from KCL schemas"

# Build Upbound platform project (generates .up/kcl/models/)
platform-build:
    cd {{ platform_dir }} && up project build
    @echo "Platform built - provider models generated in .up/kcl/models/"

# Full platform build: generate XRDs from schemas, then build with Upbound
platform-full-build:
    just platform-gen-xrds
    just platform-build
    @echo "Full platform build complete"

# Run KCL tests for platform schemas (deprecated, use platform-test-unit)
platform-test-schemas:
    cd {{ platform_dir }}/schemas && kcl test . || true

# Run platform unit tests
platform-test-unit:
    cd {{ platform_dir }} && kcl test tests/unit/

# Validate platform KCL syntax
platform-lint:
    cd {{ platform_dir }} && kcl lint schemas/
    cd {{ platform_dir }} && kcl lint render/

# Run Upbound project locally (creates KIND cluster)
platform-run-local:
    cd {{ platform_dir }} && up project run --local --ingress

# Run Upbound composition tests (requires schemas:dev to be published)
platform-test:
    cd {{ platform_dir }} && up test run tests/*

# Full test cycle: publish schemas, then run composition tests
platform-test-full: schema-publish platform-test
    @echo "Full test cycle complete"

# Render a composition for debugging
platform-render resource example:
    cd {{ platform_dir }} && up composition render apis/x{{ resource }}s/composition.yaml --composite-resource examples/{{ example }}.yaml

# Push platform to Docker Hub
platform-push version:
    cd {{ platform_dir }} && up project push docker.io/yurikrupnik/platform:{{ version }}

# Show all XRDs from schemas (preview without writing)
platform-preview-xrds:
    cd {{ platform_dir }} && kcl run render/all_xrds.k -o yaml

# Quick check: lint + test schemas
platform-check: platform-lint platform-test-schemas
    @echo "Platform checks passed"

# Full workflow: check, generate, build
platform: platform-check platform-full-build
    @echo "Platform ready!"
