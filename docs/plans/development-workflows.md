# Development Workflows & K8s Testing Infrastructure

This document describes the development workflows, testing infrastructure, and tooling for the core team and contributors.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Development Flow                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Contributor        Core Team           CI/CD                Production      │
│  ┌─────────┐       ┌─────────┐        ┌─────────┐          ┌─────────┐     │
│  │  Fork   │──PR──▶│ Review  │──Merge─▶│ Preview │──Tests──▶│ Staging │     │
│  │  +Dev   │       │         │        │   Env   │          │         │     │
│  └─────────┘       └─────────┘        └─────────┘          └────┬────┘     │
│       │                                                          │          │
│       ▼                                                          ▼          │
│  Local Kind                                                Production      │
│  Cluster                                                   (GitOps)        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Quick Start

### For New Contributors

```bash
# Clone and setup (does everything)
git clone <repo>
cd nx-playground
nu scripts/contributor.nu setup
```

This will:
1. Check prerequisites (Docker, kubectl, bun, cargo, etc.)
2. Install dependencies
3. Generate protobuf code
4. Create local K8s cluster (kind)
5. Deploy services

### For Core Team

```bash
# Daily development
nu scripts/dev-cluster.nu cluster status    # Check cluster
nu scripts/dev-cluster.nu cluster forward   # Port forward services
nx serve zerg-api                           # Start API in dev mode

# Before PR
nu scripts/contributor.nu pr                # Lint, test, format check

# Deploy preview for testing
nu scripts/gitops.nu preview create 123     # Create preview for PR #123
```

## Scripts Reference

### `scripts/dev-cluster.nu` - Local K8s Development

| Command | Description |
|---------|-------------|
| `cluster create` | Create local kind cluster with registry |
| `cluster delete` | Delete cluster |
| `cluster deploy` | Build and deploy all services |
| `cluster forward` | Port forward services |
| `cluster status` | Show cluster status |
| `cluster test` | Run integration tests |
| `cluster logs` | Watch service logs |

### `scripts/contributor.nu` - Contributor Workflow

| Command | Description |
|---------|-------------|
| `setup` | Full setup for new contributors |
| `check prerequisites` | Verify all tools installed |
| `status` | Show project/git status |
| `test` | Run tests (--affected, --watch) |
| `pr` | Validate PR (lint, build, test) |
| `clean` | Clean up environment |

### `scripts/gitops.nu` - GitOps Deployment

| Command | Description |
|---------|-------------|
| `preview create <pr>` | Create preview environment |
| `preview delete <pr>` | Delete preview |
| `staging deploy` | Deploy to staging |
| `production promote` | Promote to production |
| `production rollback` | Rollback production |
| `status` | Show deployment status |

### `scripts/k8s-test.nu` - K8s Testing

| Command | Description |
|---------|-------------|
| `integration run` | Run integration tests |
| `e2e run` | Run Playwright E2E tests |
| `load run` | Run k6 load tests |
| `chaos run` | Run chaos experiments |
| `contract run` | Run gRPC contract tests |
| `security scan` | Run security scans |
| `ci full` | Full CI test suite |

## Environment Tiers

### 1. Local (kind)

```bash
nu scripts/dev-cluster.nu cluster create
```

- Single-node or 3-node kind cluster
- Local Docker registry (localhost:5001)
- All services running
- Best for: Development, debugging

### 2. Preview (per-PR)

```bash
nu scripts/gitops.nu preview create 123
```

- Ephemeral namespace per PR
- Auto-deleted after 24h (configurable)
- Isolated testing
- Best for: PR review, QA testing

### 3. Staging

```bash
nu scripts/gitops.nu staging deploy
```

- Production-like environment
- Integration tests run here
- Load tests before promotion
- Best for: Final validation

### 4. Production

```bash
nu scripts/gitops.nu production promote --version v1.2.3
```

- GitOps controlled (ArgoCD)
- Requires tag/version
- Rollback capability
- Best for: Live traffic

## KCL Configuration

We use [KCL](https://kcl-lang.io/) for Kubernetes configurations instead of raw YAML.

### Structure

```
k8s/kcl/
├── modules/
│   ├── rust-service.k    # Rust gRPC service schema
│   └── agent.k           # AI agent schema
└── base/
    └── services.k        # Service definitions
```

### Usage

```bash
# Generate manifests
kcl run k8s/kcl/base/services.k -D PROJECT_ID=my-project -D TAG=v1.0.0

# Apply to cluster
kcl run k8s/kcl/base/services.k | kubectl apply -f -
```

### Why KCL?

| Feature | KCL | Kustomize | Helm |
|---------|-----|-----------|------|
| Type safety | ✅ | ❌ | ❌ |
| Schema validation | ✅ | ❌ | ⚠️ |
| IDE support | ✅ | ⚠️ | ⚠️ |
| No templating | ✅ | ✅ | ❌ |
| GitOps friendly | ✅ | ✅ | ⚠️ |

## Testing Strategy

### Test Pyramid

```
            ┌─────────┐
            │   E2E   │  Few, slow, high confidence
            ├─────────┤
            │ Integration │  Some, medium speed
            ├─────────────┤
            │    Unit     │  Many, fast, focused
            └─────────────┘
```

### Test Types

| Type | Tool | When | Where |
|------|------|------|-------|
| Unit | bun test, cargo test | Every commit | CI |
| Integration | Custom | Every PR | Preview/CI |
| E2E | Playwright | Every PR | Preview |
| Load | k6 | Pre-release | Staging |
| Chaos | Litmus | Scheduled | Staging |
| Contract | buf, pact | Every PR | CI |
| Security | trivy, cargo audit | Every PR | CI |

### Running Tests

```bash
# Unit tests
nx run-many --target=test

# Integration tests
nu scripts/k8s-test.nu integration run --env local

# E2E tests
nu scripts/k8s-test.nu e2e run --env preview

# Load tests
nu scripts/k8s-test.nu load run --vus 50 --duration 5m

# Full CI suite
nu scripts/k8s-test.nu ci full
```

## CNCF Tools Used

| Tool | Purpose |
|------|---------|
| **Kubernetes** | Container orchestration |
| **kind** | Local K8s clusters |
| **NATS** | Messaging (JetStream) |
| **Prometheus** | Metrics |
| **Grafana** | Dashboards |
| **OpenTelemetry** | Tracing |
| **ArgoCD** | GitOps deployments |
| **Litmus** | Chaos engineering |
| **Trivy** | Security scanning |
| **cert-manager** | TLS certificates |
| **external-secrets** | Secret management |

## GCP Integration

| Service | Purpose |
|---------|---------|
| **GKE** | Production K8s |
| **Artifact Registry** | Container images |
| **Cloud SQL** | PostgreSQL |
| **Secret Manager** | Secrets |
| **Vertex AI** | LLM (Gemini) |
| **Agent Engine** | Serverless agents |
| **Cloud Build** | CI/CD |

## Contributing

1. **Fork & Clone**
   ```bash
   gh repo fork <repo> --clone
   cd nx-playground
   ```

2. **Setup**
   ```bash
   nu scripts/contributor.nu setup
   ```

3. **Create Branch**
   ```bash
   git checkout -b feature/my-feature
   ```

4. **Develop**
   ```bash
   nu scripts/dev-cluster.nu cluster forward  # Start services
   nx serve <project>                          # Dev server
   ```

5. **Test**
   ```bash
   nu scripts/contributor.nu test --affected
   ```

6. **Submit PR**
   ```bash
   nu scripts/contributor.nu pr  # Validates everything
   gh pr create --fill
   ```

7. **Review Preview**
   - CI creates preview environment
   - Test at `https://pr-<number>.preview.example.com`

## FAQ

**Q: How do I add a new Rust service?**
1. Create service in `apps/zerg/<name>`
2. Add proto definitions in `manifests/grpc/proto`
3. Add KCL config in `k8s/kcl/base/services.k`
4. Run `buf generate` and `kcl run`

**Q: How do I add a new agent?**
1. Create agent in `apps/agents/deployable/<name>`
2. Extend `BaseDeployableAgent`
3. Add KCL config with `agent.Agent` schema
4. Deploy with `nx deploy:gke` or `nx deploy:agent-engine`

**Q: How do I debug a failing preview?**
```bash
# Get logs
kubectl logs -n preview-pr-123 -l app=<service>

# Port forward
kubectl port-forward -n preview-pr-123 svc/<service> 8080:8080

# Shell into pod
kubectl exec -it -n preview-pr-123 deploy/<service> -- sh
```
