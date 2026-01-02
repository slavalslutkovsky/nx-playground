#!/usr/bin/env just --justfile

mod dotconfig "~/dotconfig/justfile"

default:
  just -l

host := `uname -a`

shits:
    cc *.c -o main
system-info:
  @echo "This is an {{arch()}} machine".

# ============================================================================
# Dagger CI Commands
# ============================================================================

# Run full CI locally via Dagger
dagger-ci:
    cargo run -p nx-playground-ci -- all

# Run lint checks via Dagger
dagger-lint:
    cargo run -p nx-playground-ci -- lint

# Run tests via Dagger
dagger-test:
    cargo run -p nx-playground-ci -- test

# Run build via Dagger
dagger-build:
    cargo run -p nx-playground-ci -- build

# Run release build via Dagger
dagger-build-release:
    cargo run -r -p nx-playground-ci -- build --release

# Build all containers locally (no push)
dagger-container:
    cargo run -p nx-playground-ci -- container --apps all

# Build specific app container
dagger-container-app app:
    cargo run -p nx-playground-ci -- container --apps {{app}}

# Run CI with affected detection (simulating CI mode)
dagger-affected:
    cargo run -p nx-playground-ci -- all --ci --base origin/main --head HEAD

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
    wrk -t2 -c10 -d10s --latency {{api_url_cluster}}/tasks
    @echo ""
    @echo "=== Quick Benchmark: Direct DB GET (Cluster) ==="
    wrk -t2 -c10 -d10s --latency {{api_url_cluster}}/tasks-direct
    @echo ""
    @echo "Benchmark complete!"

# Quick benchmark (10s duration, lighter load) - Local
bench-tasks-quick:
    @echo "=== Quick Benchmark: gRPC GET (Local) ==="
    wrk -t2 -c10 -d10s --latency {{api_url_local}}/tasks
    @echo ""
    @echo "=== Quick Benchmark: Direct DB GET (Local) ==="
    wrk -t2 -c10 -d10s --latency {{api_url_local}}/tasks-direct

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
# Fleet Management - Multi-Cluster Kubernetes Operations
# ============================================================================

# Fleet configuration
fleet_repo := "https://github.com/yourorg/fleet-repo"
hub_context := "hub-cluster"

# --- Hub Cluster Setup ---

# Install Rancher on hub cluster
fleet-install-rancher:
    @echo "Installing Rancher on hub cluster..."
    helm repo add rancher-stable https://releases.rancher.com/server-charts/stable
    helm repo update
    kubectl create namespace cattle-system --dry-run=client -o yaml | kubectl apply -f -
    helm upgrade --install rancher rancher-stable/rancher \
        --namespace cattle-system \
        --set hostname=rancher.local \
        --set bootstrapPassword=admin \
        --set replicas=1

# Install ArgoCD on hub cluster
fleet-install-argocd:
    @echo "Installing ArgoCD..."
    kubectl create namespace argocd --dry-run=client -o yaml | kubectl apply -f -
    kubectl apply -n argocd -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml
    @echo "Waiting for ArgoCD to be ready..."
    kubectl wait --for=condition=available deployment/argocd-server -n argocd --timeout=300s
    @echo "ArgoCD installed. Get password with: kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath='{.data.password}' | base64 -d"

# Install Tailscale operator for private mesh networking
fleet-install-tailscale:
    @echo "Installing Tailscale operator..."
    helm repo add tailscale https://pkgs.tailscale.com/helmcharts
    helm repo update
    kubectl create namespace tailscale --dry-run=client -o yaml | kubectl apply -f -
    @echo "Set TS_OAUTH_CLIENT_ID and TS_OAUTH_CLIENT_SECRET env vars, then run:"
    @echo "helm install tailscale-operator tailscale/tailscale-operator -n tailscale --set oauth.clientId=\$TS_OAUTH_CLIENT_ID --set oauth.clientSecret=\$TS_OAUTH_CLIENT_SECRET"

# Install Fleet (Rancher's GitOps engine)
fleet-install-fleet:
    @echo "Installing Fleet..."
    helm repo add fleet https://rancher.github.io/fleet-helm-charts/
    helm repo update
    helm upgrade --install fleet-crd fleet/fleet-crd -n cattle-fleet-system --create-namespace
    helm upgrade --install fleet fleet/fleet -n cattle-fleet-system

# Install full hub stack (Rancher + ArgoCD + Fleet)
fleet-install-hub: fleet-install-rancher fleet-install-argocd fleet-install-fleet
    @echo "Hub cluster stack installed!"

# --- Cluster Management ---

# List all managed clusters
fleet-list-clusters:
    @echo "=== Fleet Clusters ==="
    kubectl get clusters.fleet.cattle.io -A 2>/dev/null || echo "No Fleet clusters found"
    @echo ""
    @echo "=== ArgoCD Clusters ==="
    argocd cluster list 2>/dev/null || echo "ArgoCD not configured or not logged in"

# Get cluster status
fleet-cluster-status cluster:
    @echo "=== Cluster: {{cluster}} ==="
    kubectl get cluster {{cluster}} -n fleet-default -o yaml 2>/dev/null || echo "Cluster not found in Fleet"

# Register a new cluster to Fleet (generates import command)
fleet-register-cluster name:
    @echo "Generating import command for cluster: {{name}}"
    @echo "Run this on the target cluster:"
    @echo "curl -sfL https://rancher.local/v3/import/{{name}}.yaml | kubectl apply -f -"

# Add cluster to ArgoCD
fleet-add-argocd-cluster context name:
    argocd cluster add {{context}} --name {{name}}

# --- GitOps & Deployments ---

# Apply Fleet GitRepo (connects fleet to git repository)
fleet-apply-gitrepo:
    @echo "Applying Fleet GitRepo..."
    @echo 'apiVersion: fleet.cattle.io/v1alpha1\nkind: GitRepo\nmetadata:\n  name: fleet-infra\n  namespace: fleet-default\nspec:\n  repo: {{fleet_repo}}\n  branch: main\n  paths:\n    - bundles/\n  targets:\n    - name: all\n      clusterSelector: {}' | kubectl apply -f -

# Sync all Fleet bundles
fleet-sync-all:
    @echo "Forcing sync on all clusters..."
    kubectl get clusters.fleet.cattle.io -n fleet-default -o name | xargs -I {} kubectl annotate {} -n fleet-default fleet.cattle.io/force-sync=$(date +%s) --overwrite

# Sync specific cluster
fleet-sync cluster:
    kubectl annotate cluster {{cluster}} -n fleet-default fleet.cattle.io/force-sync=$(date +%s) --overwrite

# Check bundle status
fleet-bundle-status:
    @echo "=== Bundle Status ==="
    kubectl get bundles -A
    @echo ""
    @echo "=== Bundle Deployments ==="
    kubectl get bundledeployments -A

# ArgoCD sync all apps
fleet-argocd-sync-all:
    argocd app list -o name | xargs -I {} argocd app sync {}

# ArgoCD sync apps by label
fleet-argocd-sync-env env:
    argocd app sync -l env={{env}}

# --- Monitoring & Observability ---

# Install Thanos for multi-cluster metrics
fleet-install-thanos:
    @echo "Installing Thanos..."
    helm repo add bitnami https://charts.bitnami.com/bitnami
    helm repo update
    kubectl create namespace monitoring --dry-run=client -o yaml | kubectl apply -f -
    helm upgrade --install thanos bitnami/thanos \
        --namespace monitoring \
        --set query.enabled=true \
        --set queryFrontend.enabled=true \
        --set compactor.enabled=true \
        --set storegateway.enabled=true \
        --set receive.enabled=true

# Get fleet-wide metrics overview
fleet-metrics:
    @echo "=== Cluster Health ==="
    kubectl get nodes -A --context={{hub_context}} 2>/dev/null || kubectl get nodes
    @echo ""
    @echo "=== Pod Status Across Clusters ==="
    kubectl get pods -A --field-selector=status.phase!=Running 2>/dev/null | head -20

# Check policy violations (OPA/Kyverno)
fleet-policy-violations:
    @echo "=== OPA Gatekeeper Violations ==="
    kubectl get constraints -A 2>/dev/null || echo "No Gatekeeper installed"
    @echo ""
    @echo "=== Kyverno Policy Reports ==="
    kubectl get policyreports -A 2>/dev/null || echo "No Kyverno installed"

# --- Security ---

# Install OPA Gatekeeper
fleet-install-gatekeeper:
    @echo "Installing OPA Gatekeeper..."
    kubectl apply -f https://raw.githubusercontent.com/open-policy-agent/gatekeeper/master/deploy/gatekeeper.yaml

# Install Kyverno
fleet-install-kyverno:
    @echo "Installing Kyverno..."
    helm repo add kyverno https://kyverno.github.io/kyverno/
    helm repo update
    helm upgrade --install kyverno kyverno/kyverno -n kyverno --create-namespace

# Audit RBAC for a user/service account
fleet-audit-rbac user:
    kubectl auth can-i --list --as={{user}}

# Check network policies
fleet-network-policies:
    kubectl get networkpolicies -A

# --- Patching & Updates ---

# Apply emergency patch to all clusters
fleet-apply-patch patch_name:
    @echo "Applying patch: {{patch_name}} to all clusters..."
    kubectl apply -f patches/{{patch_name}}/
    just fleet-sync-all

# Rolling restart across all clusters (use with caution)
fleet-rolling-restart namespace deployment:
    @echo "Rolling restart {{deployment}} in {{namespace}} across all clusters..."
    @echo "This will restart pods in all managed clusters!"
    @read -p "Are you sure? (y/N) " confirm && [ "$$confirm" = "y" ] || exit 1
    kubectl get clusters.fleet.cattle.io -n fleet-default -o jsonpath='{.items[*].metadata.name}' | \
        xargs -I {} kubectl rollout restart deployment/{{deployment}} -n {{namespace}} --context={}

# --- DigitalOcean Specific ---

# Create DOKS cluster with Pulumi
fleet-create-doks-cluster name region="nyc3":
    @echo "Creating DOKS cluster: {{name}} in {{region}}..."
    cd infra/pulumi && pulumi up --stack {{name}} \
        -c cluster:name={{name}} \
        -c cluster:region={{region}}

# Create DOKS cluster with doctl
fleet-create-doks name region="nyc3" size="s-4vcpu-8gb" nodes="3":
    doctl kubernetes cluster create {{name}} \
        --region {{region}} \
        --version latest \
        --size {{size}} \
        --count {{nodes}} \
        --ha \
        --auto-upgrade \
        --surge-upgrade

# List DOKS clusters
fleet-list-doks:
    doctl kubernetes cluster list

# Get DOKS kubeconfig
fleet-get-doks-config name:
    doctl kubernetes cluster kubeconfig save {{name}}

# --- Civo Specific ---

# Create Civo cluster
fleet-create-civo name region="NYC1" size="g4s.kube.medium" nodes="3":
    civo kubernetes create {{name}} \
        --size={{size}} \
        --nodes={{nodes}} \
        --region={{region}} \
        --wait

# List Civo clusters
fleet-list-civo:
    civo kubernetes list

# Get Civo kubeconfig
fleet-get-civo-config name:
    civo kubernetes config {{name}} --save

# ============================================================================
# Tilt - Local Development Environment
# ============================================================================

# Start Tilt with default k8s mode
tilt:
    tilt up

# Run in local mode (no K8s, cargo/bun directly)
tilt-local:
    tilt up -- --mode=local

# Run full stack (apps + all databases in K8s)
tilt-full:
    tilt up -- --mode=full

# Run with specific databases only
tilt-dbs *args:
    tilt up -- --mode=full --databases={{args}}

# Run with specific apps only
tilt-apps *args:
    tilt up -- --apps={{args}}

# Export all K8s manifests to ./dist/k8s/
tilt-export:
    tilt up -- --export

# Run minimal: just zerg-api + postgres + redis
tilt-minimal:
    tilt up -- --apps=zerg-api,zerg-tasks --databases=postgres,redis

# Tilt down - stop all resources
tilt-down:
    tilt down

# Show Tilt resources status
tilt-status:
    tilt get resources

# Trigger specific resource
tilt-trigger resource:
    tilt trigger {{resource}}

# Open Tilt UI in browser
tilt-ui:
    @echo "Tilt UI: http://localhost:10350"
    open http://localhost:10350

# Run checks in parallel via Tilt
tilt-check:
    tilt trigger check-rust &
    tilt trigger lint-all &
    wait

# Install dotconfig resources via Tilt
tilt-setup:
    tilt trigger install-dotconfig
    tilt trigger install-cargo-tools
    tilt trigger install-brew-deps

# --- Utilities ---

# Switch kubectl context
fleet-use-context context:
    kubectl config use-context {{context}}

# List all contexts
fleet-list-contexts:
    kubectl config get-contexts

# Port forward to Rancher UI
fleet-rancher-ui:
    @echo "Rancher UI available at https://localhost:8443"
    kubectl port-forward -n cattle-system svc/rancher 8443:443

# Port forward to ArgoCD UI
fleet-argocd-ui:
    @echo "ArgoCD UI available at https://localhost:8080"
    @echo "Username: admin"
    @echo "Password: $(kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath='{.data.password}' | base64 -d)"
    kubectl port-forward -n argocd svc/argocd-server 8080:443

# Port forward to Grafana
fleet-grafana-ui:
    @echo "Grafana available at http://localhost:3000"
    kubectl port-forward -n monitoring svc/grafana 3000:80

# Show fleet management help
fleet-help:
    @echo "=== Fleet Management Commands ==="
    @echo ""
    @echo "Setup:"
    @echo "  just fleet-install-hub          - Install full hub stack"
    @echo "  just fleet-install-rancher      - Install Rancher"
    @echo "  just fleet-install-argocd       - Install ArgoCD"
    @echo "  just fleet-install-tailscale    - Install Tailscale"
    @echo ""
    @echo "Clusters:"
    @echo "  just fleet-list-clusters        - List all managed clusters"
    @echo "  just fleet-create-doks <name>   - Create DigitalOcean cluster"
    @echo "  just fleet-create-civo <name>   - Create Civo cluster"
    @echo ""
    @echo "GitOps:"
    @echo "  just fleet-sync-all             - Sync all clusters"
    @echo "  just fleet-sync <cluster>       - Sync specific cluster"
    @echo "  just fleet-bundle-status        - Check bundle status"
    @echo ""
    @echo "Security:"
    @echo "  just fleet-policy-violations    - Check policy violations"
    @echo "  just fleet-audit-rbac <user>    - Audit RBAC permissions"
    @echo ""
    @echo "UI Access:"
    @echo "  just fleet-rancher-ui           - Port forward to Rancher"
    @echo "  just fleet-argocd-ui            - Port forward to ArgoCD"
    @echo "  just fleet-grafana-ui           - Port forward to Grafana"
