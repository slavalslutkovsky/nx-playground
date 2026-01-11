# Deployment Guide

This guide covers deploying the nx-playground platform to Kubernetes.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Local Development](#local-development)
- [Kubernetes Deployment](#kubernetes-deployment)
- [Environment Configuration](#environment-configuration)
- [Secrets Management](#secrets-management)
- [Scaling](#scaling)
- [Monitoring](#monitoring)
- [Rollback Procedures](#rollback-procedures)

---

## Prerequisites

### Required Tools

```bash
# Kubernetes CLI
brew install kubectl

# Kustomize
brew install kustomize

# Helm (for some dependencies)
brew install helm

# Docker
brew install --cask docker

# Optional: k9s for cluster management
brew install derailed/k9s/k9s
```

### Cluster Requirements

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| Nodes | 3 | 5+ |
| CPU/Node | 4 cores | 8 cores |
| Memory/Node | 8 GB | 16 GB |
| Kubernetes | 1.28+ | 1.29+ |

---

## Local Development

### Start Infrastructure

```bash
# Start databases and message broker
docker-compose -f manifests/dockers/compose.yaml up -d

# Verify services
docker-compose ps
```

### Run Services

```bash
# Terminal 1: API Gateway
cargo run -p zerg_api

# Terminal 2: Tasks Service
cargo run -p zerg_tasks

# Terminal 3: Vector Service
cargo run -p zerg_vector

# Terminal 4: Agent Gateway
cd apps/agents/gateway && bun run dev
```

### Run Migrations

```bash
just _migration
# or
sqlx migrate run --database-url $DATABASE_URL --source manifests/migrations/postgres/
```

---

## Kubernetes Deployment

### 1. Namespace Setup

```bash
# Create namespaces
kubectl apply -f k8s/core/base/namespace.yaml

# Verify
kubectl get namespaces
```

### 2. Deploy Core Infrastructure

```bash
# Development environment
kubectl apply -k k8s/core/overlays/dev

# Production environment
kubectl apply -k k8s/core/overlays/prod
```

### 3. Deploy External Secrets

```bash
# Vault integration
kubectl apply -k k8s/external-secrets/overlays/prod

# Verify secrets are syncing
kubectl get externalsecrets -n zerg
```

### 4. Deploy Applications

```bash
# Deploy all Zerg services
kubectl apply -k apps/zerg/api/k8s/kustomize/overlays/dev
kubectl apply -k apps/zerg/tasks/k8s/kustomize/overlays/dev
kubectl apply -k apps/zerg/vector/k8s/kustomize/overlays/dev
kubectl apply -k apps/zerg/email-nats/k8s/kustomize/overlays/dev

# Deploy Agent Gateway
kubectl apply -k apps/agents/gateway/k8s/overlays/dev
```

### 5. Apply Hardening

```bash
# Network policies, PDBs, resource quotas
kubectl apply -k k8s/hardening/base

# Verify network policies
kubectl get networkpolicies -n zerg
```

### 6. Deploy Observability

```bash
# Prometheus, Grafana, OpenTelemetry
kubectl apply -k k8s/observability/overlays/dev

# Access Grafana
kubectl port-forward svc/grafana 3000:3000 -n monitoring
```

---

## Environment Configuration

### ConfigMaps

```yaml
# k8s/core/base/configmaps/shared-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: zerg-shared-config
  namespace: zerg
data:
  DB_MAX_CONNECTIONS: "10"
  DB_MIN_CONNECTIONS: "2"
  POSTGRES_HOST: "postgres.dbs.svc.cluster.local"
  REDIS_HOST: "redis://redis.dbs.svc.cluster.local"
  NATS_URL: "nats://nats.dbs.svc.cluster.local:4222"
```

### Environment-Specific Patches

```yaml
# k8s/core/overlays/prod/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

resources:
  - ../../base

patches:
  - path: configmap-patch.yaml
  - path: ingress-patch.yaml

configMapGenerator:
  - name: zerg-shared-config
    behavior: merge
    literals:
      - DB_MAX_CONNECTIONS=50
      - LOG_LEVEL=warn
```

---

## Secrets Management

### Vault Setup

```bash
# Enable Kubernetes auth
vault auth enable kubernetes

# Configure Kubernetes auth
vault write auth/kubernetes/config \
  kubernetes_host="https://$KUBERNETES_HOST:443"

# Create policy
vault policy write zerg-secrets - <<EOF
path "secret/data/zerg/*" {
  capabilities = ["read"]
}
path "aws/creds/ses-sender" {
  capabilities = ["read"]
}
EOF

# Create role
vault write auth/kubernetes/role/external-secrets \
  bound_service_account_names=external-secrets \
  bound_service_account_namespaces=zerg \
  policies=zerg-secrets \
  ttl=1h
```

### Required Secrets

| Secret Path | Keys | Description |
|-------------|------|-------------|
| `secret/data/zerg/database` | `DATABASE_USER`, `DATABASE_PASSWORD`, `DATABASE_NAME` | PostgreSQL credentials |
| `secret/data/zerg/auth` | `JWT_SECRET` | JWT signing key (min 32 chars) |
| `secret/data/zerg/oauth` | `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, `GITHUB_CLIENT_ID`, `GITHUB_CLIENT_SECRET` | OAuth credentials |
| `secret/data/zerg/email` | `SENDGRID_API_KEY` | Email service |
| `secret/data/zerg/llm` | `ANTHROPIC_API_KEY`, `OPENAI_API_KEY` | LLM providers |
| `secret/data/zerg/braintrust` | `BRAINTRUST_API_KEY` | Agent tracing |

---

## Scaling

### Horizontal Pod Autoscaler

```yaml
# Already configured in deployment manifests
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: zerg-api-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: zerg-api
  minReplicas: 2
  maxReplicas: 10
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
```

### KEDA Event-Driven Scaling

```bash
# Install KEDA
kubectl apply -f https://github.com/kedacore/keda/releases/download/v2.12.0/keda-2.12.0.yaml

# Apply KEDA scalers
kubectl apply -f k8s/hardening/base/keda-scalers.yaml
```

### Manual Scaling

```bash
# Scale deployment
kubectl scale deployment zerg-api --replicas=5 -n zerg

# Scale via patch
kubectl patch deployment zerg-api -n zerg -p '{"spec":{"replicas":5}}'
```

---

## Monitoring

### Access Dashboards

```bash
# Grafana
kubectl port-forward svc/grafana 3000:3000 -n monitoring
# Open http://localhost:3000

# Prometheus
kubectl port-forward svc/prometheus 9090:9090 -n monitoring
# Open http://localhost:9090
```

### Key Metrics

```promql
# API request rate
sum(rate(http_requests_total{service="zerg-api"}[5m]))

# Error rate
sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m]))

# Latency p95
histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket[5m])) by (le))

# Agent health
agent_health{agent="rag"}
```

### Health Checks

```bash
# Check all pods
kubectl get pods -n zerg

# Check pod logs
kubectl logs -f deployment/zerg-api -n zerg

# Check events
kubectl get events -n zerg --sort-by='.lastTimestamp'

# Describe pod
kubectl describe pod <pod-name> -n zerg
```

---

## Rollback Procedures

### Deployment Rollback

```bash
# View rollout history
kubectl rollout history deployment/zerg-api -n zerg

# Rollback to previous version
kubectl rollout undo deployment/zerg-api -n zerg

# Rollback to specific revision
kubectl rollout undo deployment/zerg-api --to-revision=2 -n zerg

# Check rollout status
kubectl rollout status deployment/zerg-api -n zerg
```

### Database Rollback

```bash
# List available backups
velero backup get

# Restore from backup
velero restore create --from-backup <backup-name>
```

### Emergency Procedures

```bash
# Scale to zero (emergency stop)
kubectl scale deployment --all --replicas=0 -n zerg

# Drain node for maintenance
kubectl drain <node-name> --ignore-daemonsets --delete-emptydir-data

# Cordon node (prevent new pods)
kubectl cordon <node-name>
```

---

## CI/CD Integration

### GitHub Actions Example

```yaml
# .github/workflows/deploy.yml
name: Deploy

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build images
        run: |
          docker build -t zerg-api:${{ github.sha }} -f apps/zerg/api/Dockerfile .
          docker build -t agent-gateway:${{ github.sha }} -f apps/agents/gateway/Dockerfile .

      - name: Push to registry
        run: |
          docker push $REGISTRY/zerg-api:${{ github.sha }}
          docker push $REGISTRY/agent-gateway:${{ github.sha }}

      - name: Deploy to Kubernetes
        run: |
          kubectl set image deployment/zerg-api zerg-api=$REGISTRY/zerg-api:${{ github.sha }} -n zerg
          kubectl set image deployment/agent-gateway agent-gateway=$REGISTRY/agent-gateway:${{ github.sha }} -n zerg
          kubectl rollout status deployment/zerg-api -n zerg
```

### FluxCD GitOps

```yaml
# k8s/gitops/base/flux-sync.yaml
apiVersion: kustomize.toolkit.fluxcd.io/v1
kind: Kustomization
metadata:
  name: zerg-apps
  namespace: flux-system
spec:
  interval: 10m
  path: ./k8s/apps/overlays/prod
  prune: true
  sourceRef:
    kind: GitRepository
    name: flux-system
```

---

## Pre-Deployment Checklist

- [ ] All tests passing (`cargo test --workspace`)
- [ ] Docker images built and pushed
- [ ] Secrets configured in Vault
- [ ] ConfigMaps updated for environment
- [ ] Database migrations run
- [ ] Resource quotas reviewed
- [ ] Network policies applied
- [ ] Monitoring dashboards accessible
- [ ] Alerting rules configured
- [ ] Rollback procedure documented
- [ ] Team notified of deployment

---

## Related Documentation

- [Architecture Overview](./ARCHITECTURE.md)
- [Agents Guide](./AGENTS.md)
- [Troubleshooting](./TROUBLESHOOTING.md)
