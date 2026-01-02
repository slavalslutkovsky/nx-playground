# Kubernetes Fleet Management: Multi-Cluster Operations Guide

## Table of Contents

1. [Overview](#overview)
2. [Architecture Patterns](#architecture-patterns)
3. [Fleet Management Tools](#fleet-management-tools)
4. [What, Who, Where - Command & Control](#what-who-where---command--control)
5. [Deployment Strategies](#deployment-strategies)
6. [Security Best Practices](#security-best-practices)
7. [Monitoring & Observability](#monitoring--observability)
8. [Network Architecture](#network-architecture)
9. [Implementation Guide](#implementation-guide)

---

## Overview

### Fleet Management Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                     KUBERNETES FLEET MANAGEMENT                                  │
└─────────────────────────────────────────────────────────────────────────────────┘

                         ┌─────────────────────────┐
                         │    MANAGEMENT PLANE     │
                         │    (Your Network)       │
                         │                         │
                         │  ┌─────────────────┐   │
                         │  │  Fleet Manager  │   │
                         │  │  (Rancher/Fleet │   │
                         │  │   /Anthos)      │   │
                         │  └────────┬────────┘   │
                         │           │            │
                         │  ┌────────▼────────┐   │
                         │  │    GitOps       │   │
                         │  │  (ArgoCD/Flux)  │   │
                         │  └────────┬────────┘   │
                         │           │            │
                         │  ┌────────▼────────┐   │
                         │  │   Policy Engine │   │
                         │  │  (OPA/Kyverno)  │   │
                         │  └────────┬────────┘   │
                         └───────────┼────────────┘
                                     │
         ┌───────────────────────────┼───────────────────────────┐
         │                           │                           │
         ▼                           ▼                           ▼
┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
│   CLIENT A      │       │   CLIENT B      │       │   CLIENT C      │
│   CLUSTERS      │       │   CLUSTERS      │       │   CLUSTERS      │
│                 │       │                 │       │                 │
│ ┌─────────────┐ │       │ ┌─────────────┐ │       │ ┌─────────────┐ │
│ │ Production  │ │       │ │ Production  │ │       │ │ Production  │ │
│ │  Cluster    │ │       │ │  Cluster    │ │       │ │  Cluster    │ │
│ └─────────────┘ │       │ └─────────────┘ │       │ └─────────────┘ │
│ ┌─────────────┐ │       │ ┌─────────────┐ │       │ ┌─────────────┐ │
│ │  Staging    │ │       │ │  Staging    │ │       │ │  Staging    │ │
│ │  Cluster    │ │       │ │  Cluster    │ │       │ │  Cluster    │ │
│ └─────────────┘ │       │ └─────────────┘ │       │ └─────────────┘ │
│ ┌─────────────┐ │       │ ┌─────────────┐ │       │                 │
│ │    Edge     │ │       │ │    Dev      │ │       │                 │
│ │  Clusters   │ │       │ │  Cluster    │ │       │                 │
│ └─────────────┘ │       │ └─────────────┘ │       │                 │
└─────────────────┘       └─────────────────┘       └─────────────────┘
     │                          │                          │
     └──────────────────────────┴──────────────────────────┘
                                │
                    ┌───────────▼───────────┐
                    │  UNIFIED MONITORING   │
                    │  (Prometheus/Thanos   │
                    │   Grafana/Loki)       │
                    └───────────────────────┘
```

---

## Architecture Patterns

### Hub-Spoke Model (Recommended)

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                          HUB-SPOKE ARCHITECTURE                                  │
└─────────────────────────────────────────────────────────────────────────────────┘

                              ┌─────────────────────┐
                              │     HUB CLUSTER     │
                              │  (Management Plane) │
                              │                     │
                              │ ┌─────────────────┐ │
                              │ │ Fleet Controller│ │
                              │ │    (Rancher)    │ │
                              │ └─────────────────┘ │
                              │ ┌─────────────────┐ │
                              │ │  ArgoCD/Flux    │ │
                              │ │   (GitOps)      │ │
                              │ └─────────────────┘ │
                              │ ┌─────────────────┐ │
                              │ │ Vault (Secrets) │ │
                              │ └─────────────────┘ │
                              │ ┌─────────────────┐ │
                              │ │  Thanos/Mimir   │ │
                              │ │  (Metrics Hub)  │ │
                              │ └─────────────────┘ │
                              └──────────┬──────────┘
                                         │
            ┌────────────────────────────┼────────────────────────────┐
            │                            │                            │
            ▼                            ▼                            ▼
   ┌─────────────────┐         ┌─────────────────┐         ┌─────────────────┐
   │  SPOKE CLUSTER  │         │  SPOKE CLUSTER  │         │  SPOKE CLUSTER  │
   │   (Client A)    │         │   (Client B)    │         │   (Client C)    │
   │                 │         │                 │         │                 │
   │ ┌─────────────┐ │         │ ┌─────────────┐ │         │ ┌─────────────┐ │
   │ │Fleet Agent  │ │         │ │Fleet Agent  │ │         │ │Fleet Agent  │ │
   │ └─────────────┘ │         │ └─────────────┘ │         │ └─────────────┘ │
   │ ┌─────────────┐ │         │ ┌─────────────┐ │         │ ┌─────────────┐ │
   │ │ArgoCD Agent │ │         │ │ArgoCD Agent │ │         │ │ArgoCD Agent │ │
   │ └─────────────┘ │         │ └─────────────┘ │         │ └─────────────┘ │
   │ ┌─────────────┐ │         │ ┌─────────────┐ │         │ ┌─────────────┐ │
   │ │Prom Agent   │ │         │ │Prom Agent   │ │         │ │Prom Agent   │ │
   │ └─────────────┘ │         │ └─────────────┘ │         │ └─────────────┘ │
   └─────────────────┘         └─────────────────┘         └─────────────────┘
```

### Control Plane Options

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                        CONTROL PLANE OPTIONS                                      │
├──────────────────┬───────────────┬───────────────┬───────────────┬───────────────┤
│                  │   Rancher     │  Fleet (SUSE) │   ArgoCD      │  Anthos       │
├──────────────────┼───────────────┼───────────────┼───────────────┼───────────────┤
│ Cluster Import   │ ✓ Any K8s     │ ✓ Any K8s     │ ✓ Any K8s     │ GKE focused   │
│ GitOps           │ Fleet built-in│ ✓ Native      │ ✓ Native      │ Config Sync   │
│ UI               │ ✓ Full UI     │ Basic         │ ✓ Good UI     │ ✓ GCP Console │
│ Multi-tenancy    │ ✓ Projects    │ Workspaces    │ Projects      │ ✓ Teams       │
│ RBAC             │ ✓ Fine-grained│ ✓ Namespace   │ ✓ RBAC        │ ✓ IAM         │
│ Cost             │ Free/Paid     │ Free          │ Free          │ $$$           │
│ Self-hosted      │ ✓             │ ✓             │ ✓             │ ✗             │
└──────────────────┴───────────────┴───────────────┴───────────────┴───────────────┘
```

---

## Fleet Management Tools

### Tool Comparison

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                    FLEET MANAGEMENT TOOL SELECTION                               │
└─────────────────────────────────────────────────────────────────────────────────┘

                    ┌─────────────────────────────────┐
                    │      What do you need?          │
                    └────────────────┬────────────────┘
                                     │
         ┌───────────────────────────┼───────────────────────────┐
         │                           │                           │
         ▼                           ▼                           ▼
┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
│ Full Platform   │       │  GitOps Only    │       │ Enterprise      │
│ (UI + Mgmt)     │       │  (Deployments)  │       │ (Support + SLA) │
└────────┬────────┘       └────────┬────────┘       └────────┬────────┘
         │                         │                         │
         ▼                         ▼                         ▼
┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
│    Rancher      │       │    ArgoCD       │       │ Rancher Prime   │
│    (Free)       │       │    + Fleet      │       │ OpenShift       │
│                 │       │    (Free)       │       │ Tanzu           │
│ • Full UI       │       │                 │       │ Anthos          │
│ • Cluster CRUD  │       │ • Pure GitOps   │       │                 │
│ • App Catalog   │       │ • Lightweight   │       │ • 24/7 Support  │
│ • Monitoring    │       │ • K8s Native    │       │ • SLAs          │
└─────────────────┘       └─────────────────┘       └─────────────────┘
```

---

## What, Who, Where - Command & Control

### The 3W Model

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           WHAT - WHO - WHERE                                     │
│                        Command & Control Matrix                                  │
└─────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────┐
│  WHAT (Resources)                                                                │
│  ─────────────────                                                               │
│                                                                                  │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐    │
│  │  Applications │  │  Policies     │  │  Secrets      │  │  Configs      │    │
│  │  (Helm/Kust.) │  │  (OPA/Kyver.) │  │  (Vault/ESO)  │  │  (ConfigMaps) │    │
│  └───────────────┘  └───────────────┘  └───────────────┘  └───────────────┘    │
│                                                                                  │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐    │
│  │  CRDs         │  │  RBAC Rules   │  │  Network Pol. │  │  Patches      │    │
│  │               │  │               │  │               │  │  (Updates)    │    │
│  └───────────────┘  └───────────────┘  └───────────────┘  └───────────────┘    │
└─────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────┐
│  WHO (Identity & Access)                                                         │
│  ───────────────────────                                                         │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                         IDENTITY HIERARCHY                               │    │
│  │                                                                          │    │
│  │  Platform Admin ──► Tenant Admin ──► Developer ──► Read-Only            │    │
│  │       │                  │                │             │                │    │
│  │       ▼                  ▼                ▼             ▼                │    │
│  │  All Clusters      Client Clusters    Namespaces    View Only           │    │
│  │  All Tenants       All Envs          Dev/Staging    No Secrets          │    │
│  │  All Secrets       Limited Secrets   No Prod       Logs Only            │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                  │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐    │
│  │  SSO/OIDC     │  │  Service Accts│  │  API Tokens   │  │  Cert Auth    │    │
│  │  (Human)      │  │  (Automation) │  │  (CI/CD)      │  │  (mTLS)       │    │
│  └───────────────┘  └───────────────┘  └───────────────┘  └───────────────┘    │
└─────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────┐
│  WHERE (Target Clusters)                                                         │
│  ───────────────────────                                                         │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                      CLUSTER SELECTORS                                   │    │
│  │                                                                          │    │
│  │  By Label:        env=production, client=acme, region=us-east           │    │
│  │  By Name:         cluster-prod-*, *-staging-*                           │    │
│  │  By Group:        client-a/*, edge-clusters/*                           │    │
│  │  By Annotation:   tier=critical, compliance=pci                         │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                  │
│  ┌───────────────────────────────────────────────────────────────────────────┐  │
│  │  CLIENT A                │  CLIENT B                │  CLIENT C           │  │
│  │  ├── prod-us-east        │  ├── prod-eu-west        │  ├── prod-asia      │  │
│  │  ├── prod-us-west        │  ├── staging             │  └── staging        │  │
│  │  ├── staging             │  └── dev                 │                     │  │
│  │  └── edge-retail-*       │                          │                     │  │
│  └───────────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### GitOps Repository Structure

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                      GITOPS REPOSITORY STRUCTURE                                 │
└─────────────────────────────────────────────────────────────────────────────────┘

fleet-repo/
├── README.md
├── fleet.yaml                          # Global fleet configuration
│
├── base/                               # Shared base configurations
│   ├── monitoring/
│   │   ├── prometheus/
│   │   ├── grafana/
│   │   └── alertmanager/
│   ├── security/
│   │   ├── falco/
│   │   ├── opa-gatekeeper/
│   │   └── network-policies/
│   ├── logging/
│   │   ├── loki/
│   │   └── fluentbit/
│   └── common/
│       ├── namespaces/
│       └── rbac/
│
├── clusters/                           # Cluster-specific overrides
│   ├── _templates/                     # Cluster templates
│   │   ├── production.yaml
│   │   ├── staging.yaml
│   │   └── edge.yaml
│   │
│   ├── client-a/
│   │   ├── prod-us-east/
│   │   │   ├── cluster.yaml           # Cluster metadata
│   │   │   ├── kustomization.yaml
│   │   │   └── values.yaml            # Helm value overrides
│   │   ├── prod-us-west/
│   │   └── staging/
│   │
│   ├── client-b/
│   │   ├── prod-eu-west/
│   │   └── dev/
│   │
│   └── client-c/
│       └── prod-asia/
│
├── bundles/                            # Deployment bundles (Fleet)
│   ├── core-stack/                     # Always deployed to all clusters
│   │   ├── fleet.yaml
│   │   └── kustomization.yaml
│   ├── monitoring-stack/
│   │   ├── fleet.yaml
│   │   └── kustomization.yaml
│   └── app-stack/
│       ├── fleet.yaml
│       └── kustomization.yaml
│
├── policies/                           # OPA/Kyverno policies
│   ├── require-labels.yaml
│   ├── deny-privileged.yaml
│   ├── require-resource-limits.yaml
│   └── restrict-registries.yaml
│
└── patches/                            # Emergency patches
    ├── cve-2024-xxxx/
    │   ├── fleet.yaml
    │   └── patch.yaml
    └── security-update-01/
        ├── fleet.yaml
        └── patch.yaml
```

### Fleet Bundle Example

```yaml
# bundles/core-stack/fleet.yaml
defaultNamespace: kube-system

# WHAT: Define what to deploy
helm:
  releaseName: core-stack
  chart: ./charts/core-stack
  values:
    monitoring:
      enabled: true
    logging:
      enabled: true

# WHERE: Target clusters by labels
targets:
  # All production clusters
  - name: production
    clusterSelector:
      matchLabels:
        env: production
    helm:
      values:
        replicas: 3
        resources:
          limits:
            memory: 2Gi

  # All staging clusters
  - name: staging
    clusterSelector:
      matchLabels:
        env: staging
    helm:
      values:
        replicas: 1
        resources:
          limits:
            memory: 512Mi

  # Specific client clusters
  - name: client-a-edge
    clusterSelector:
      matchLabels:
        client: client-a
        type: edge
    helm:
      values:
        lightweight: true
        resources:
          limits:
            memory: 256Mi

# WHO: Define which groups can modify
targetCustomizations:
  - name: client-a-override
    clusterGroup: client-a
    yaml:
      overlays:
        - content: |
            apiVersion: v1
            kind: ConfigMap
            metadata:
              name: client-config
            data:
              CLIENT_ID: "client-a"
```

---

## Deployment Strategies

### Rolling Deployment Across Fleet

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                    FLEET-WIDE ROLLING DEPLOYMENT                                 │
└─────────────────────────────────────────────────────────────────────────────────┘

                              TIME ──────────────────────────────────►

Wave 1 (Canary)          Wave 2 (Staging)         Wave 3 (Production)
────────────────         ────────────────         ──────────────────

┌─────────────┐
│  1 Cluster  │          ┌─────────────┐
│  (Canary)   │          │ All Staging │          ┌─────────────────┐
│             │    ───►  │  Clusters   │    ───►  │ Prod by Region  │
│ • Validate  │          │             │          │                 │
│ • Monitor   │          │ • 10% users │          │ • us-east first │
│ • 1hr wait  │          │ • 4hr wait  │          │ • then us-west  │
└─────────────┘          └─────────────┘          │ • then eu       │
                                                  │ • then asia     │
                                                  └─────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────┐
│  ROLLOUT POLICY                                                                  │
│                                                                                  │
│  maxConcurrent: 2          # Max clusters updating at once                      │
│  pauseBetweenBatches: 1h   # Wait between batches                               │
│  failureThreshold: 1       # Rollback if 1 cluster fails                        │
│  successThreshold: 95%     # Consider success if 95% pods healthy               │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Patch Deployment Strategy

```yaml
# patches/cve-2024-critical/fleet.yaml
defaultNamespace: kube-system

# Emergency patch configuration
helm:
  chart: ./security-patch
  values:
    patchVersion: "1.0.0"
    applyImmediately: true

# Progressive rollout
targets:
  # Wave 1: Canary (1 non-critical cluster)
  - name: canary
    clusterSelector:
      matchLabels:
        tier: canary
    correctDrift:
      enabled: true
      force: true

  # Wave 2: Non-production (after 30 min)
  - name: non-prod
    clusterSelector:
      matchExpressions:
        - key: env
          operator: NotIn
          values: ["production"]

  # Wave 3: Production (after validation)
  - name: production
    clusterSelector:
      matchLabels:
        env: production

# Rollout constraints
rollout:
  maxConcurrent: 3
  partitions:
    - name: canary
      maxConcurrent: 1
    - name: non-prod
      maxConcurrent: 5
    - name: production
      maxConcurrent: 2
```

---

## Security Best Practices

### Security Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                      FLEET SECURITY ARCHITECTURE                                 │
└─────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────┐
│  LAYER 1: NETWORK SECURITY                                                       │
│  ─────────────────────────                                                       │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                         PRIVATE NETWORK                                  │    │
│  │                                                                          │    │
│  │  Hub Cluster ◄───── VPN/mTLS ─────► Spoke Clusters                      │    │
│  │       │                                    │                             │    │
│  │       │         No Public API Servers      │                             │    │
│  │       │         Tailscale/WireGuard        │                             │    │
│  │       │         Network Policies           │                             │    │
│  │       ▼                                    ▼                             │    │
│  │  ┌─────────────┐                    ┌─────────────┐                     │    │
│  │  │ Bastion/VPN │                    │  Agent-only │                     │    │
│  │  │   Access    │                    │  Outbound   │                     │    │
│  │  └─────────────┘                    └─────────────┘                     │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────┐
│  LAYER 2: IDENTITY & ACCESS                                                      │
│  ──────────────────────────                                                      │
│                                                                                  │
│  ┌──────────────────────────────────────────────────────────────────────────┐   │
│  │  AUTHENTICATION                      │  AUTHORIZATION                    │   │
│  │  ──────────────                      │  ─────────────                    │   │
│  │  • OIDC/SSO (Okta, Azure AD)        │  • RBAC per Cluster              │   │
│  │  • Short-lived tokens (1hr)          │  • Namespace isolation           │   │
│  │  • MFA required for prod             │  • Tenant boundaries             │   │
│  │  • Service account per cluster       │  • Audit logging                 │   │
│  │  • No shared credentials             │  • Just-in-time access           │   │
│  └──────────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────┐
│  LAYER 3: SECRETS MANAGEMENT                                                     │
│  ───────────────────────────                                                     │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                                                                          │    │
│  │   ┌─────────────┐         ┌─────────────┐         ┌─────────────┐       │    │
│  │   │   Vault     │────────►│External Sec.│────────►│  K8s Secret │       │    │
│  │   │  (Central)  │         │  Operator   │         │  (Dynamic)  │       │    │
│  │   └─────────────┘         └─────────────┘         └─────────────┘       │    │
│  │                                                                          │    │
│  │   • Secrets never in Git          • Auto-rotation                       │    │
│  │   • Per-tenant namespaces         • Audit trail                         │    │
│  │   • Dynamic credentials           • Encryption at rest                  │    │
│  │                                                                          │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────┐
│  LAYER 4: POLICY ENFORCEMENT                                                     │
│  ───────────────────────────                                                     │
│                                                                                  │
│  ┌──────────────────────────────────────────────────────────────────────────┐   │
│  │                        OPA GATEKEEPER / KYVERNO                          │   │
│  │                                                                          │   │
│  │  DENY:                              │  REQUIRE:                          │   │
│  │  • Privileged containers            │  • Resource limits                 │   │
│  │  • Host networking                  │  • Security context               │   │
│  │  • Unknown registries               │  • Labels (owner, env, app)       │   │
│  │  • Latest tags                      │  • Network policies               │   │
│  │  • Root users                       │  • Pod disruption budgets         │   │
│  │                                                                          │   │
│  │  MUTATE:                            │  AUDIT:                            │   │
│  │  • Add default labels               │  • Log all violations             │   │
│  │  • Inject sidecars                  │  • Alert on critical              │   │
│  │  • Set resource defaults            │  • Report compliance              │   │
│  └──────────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Multi-Tenant RBAC Model

```yaml
# rbac/platform-roles.yaml
---
# Platform Admin - Full access to all clusters
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: platform-admin
rules:
  - apiGroups: ["*"]
    resources: ["*"]
    verbs: ["*"]
---
# Tenant Admin - Full access within tenant namespace
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: tenant-admin
rules:
  - apiGroups: ["", "apps", "batch", "networking.k8s.io"]
    resources: ["*"]
    verbs: ["*"]
  - apiGroups: [""]
    resources: ["secrets"]
    verbs: ["get", "list"]  # Can view but not create global secrets
---
# Developer - Limited access
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: developer
rules:
  - apiGroups: ["", "apps"]
    resources: ["pods", "deployments", "services", "configmaps"]
    verbs: ["get", "list", "watch", "create", "update", "patch"]
  - apiGroups: [""]
    resources: ["pods/log", "pods/exec"]
    verbs: ["get", "create"]
  - apiGroups: [""]
    resources: ["secrets"]
    verbs: ["get", "list"]  # Read-only secrets
---
# Read-Only Viewer
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: viewer
rules:
  - apiGroups: ["", "apps", "batch", "networking.k8s.io"]
    resources: ["*"]
    verbs: ["get", "list", "watch"]
  - apiGroups: [""]
    resources: ["secrets"]
    verbs: []  # No access to secrets
```

### Network Policies for Tenant Isolation

```yaml
# policies/tenant-isolation.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: tenant-isolation
  namespace: tenant-a  # Applied per tenant namespace
spec:
  podSelector: {}  # All pods in namespace
  policyTypes:
    - Ingress
    - Egress
  ingress:
    # Allow from same namespace
    - from:
        - namespaceSelector:
            matchLabels:
              tenant: tenant-a
    # Allow from ingress controller
    - from:
        - namespaceSelector:
            matchLabels:
              name: ingress-nginx
      ports:
        - protocol: TCP
          port: 8080
    # Allow from monitoring
    - from:
        - namespaceSelector:
            matchLabels:
              name: monitoring
      ports:
        - protocol: TCP
          port: 9090
  egress:
    # Allow DNS
    - to:
        - namespaceSelector: {}
      ports:
        - protocol: UDP
          port: 53
    # Allow same namespace
    - to:
        - namespaceSelector:
            matchLabels:
              tenant: tenant-a
    # Allow external HTTPS
    - to:
        - ipBlock:
            cidr: 0.0.0.0/0
      ports:
        - protocol: TCP
          port: 443
```

---

## Monitoring & Observability

### Multi-Cluster Monitoring Stack

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                    MULTI-CLUSTER OBSERVABILITY                                   │
└─────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────┐
│                           HUB CLUSTER                                            │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                         METRICS AGGREGATION                              │    │
│  │                                                                          │    │
│  │   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                 │    │
│  │   │   Thanos    │◄───│   Thanos    │◄───│   Thanos    │                 │    │
│  │   │   Query     │    │   Store     │    │  Compactor  │                 │    │
│  │   └──────┬──────┘    └─────────────┘    └─────────────┘                 │    │
│  │          │                                                               │    │
│  │          ▼                                                               │    │
│  │   ┌─────────────┐                                                        │    │
│  │   │  Grafana    │ ◄─── Unified dashboards for all clusters              │    │
│  │   └─────────────┘                                                        │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                           LOGS AGGREGATION                               │    │
│  │                                                                          │    │
│  │   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                 │    │
│  │   │    Loki     │◄───│   Loki      │◄───│   Loki      │                 │    │
│  │   │   Gateway   │    │  Ingester   │    │  Compactor  │                 │    │
│  │   └─────────────┘    └─────────────┘    └─────────────┘                 │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                         ALERTING                                         │    │
│  │                                                                          │    │
│  │   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                 │    │
│  │   │AlertManager │───►│  PagerDuty  │    │   Slack     │                 │    │
│  │   │  (Central)  │───►│             │    │             │                 │    │
│  │   └─────────────┘    └─────────────┘    └─────────────┘                 │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
└───────────────────────────────────────┬─────────────────────────────────────────┘
                                        │
         ┌──────────────────────────────┼──────────────────────────────┐
         │                              │                              │
         ▼                              ▼                              ▼
┌─────────────────┐          ┌─────────────────┐          ┌─────────────────┐
│  SPOKE CLUSTER  │          │  SPOKE CLUSTER  │          │  SPOKE CLUSTER  │
│                 │          │                 │          │                 │
│ ┌─────────────┐ │          │ ┌─────────────┐ │          │ ┌─────────────┐ │
│ │ Prometheus  │ │          │ │ Prometheus  │ │          │ │ Prometheus  │ │
│ │ + Thanos    │─┼──────────┼─│ + Thanos    │─┼──────────┼─│ + Thanos    │ │
│ │   Sidecar   │ │ Remote   │ │   Sidecar   │ │ Remote   │ │   Sidecar   │ │
│ └─────────────┘ │ Write    │ └─────────────┘ │ Write    │ └─────────────┘ │
│                 │          │                 │          │                 │
│ ┌─────────────┐ │          │ ┌─────────────┐ │          │ ┌─────────────┐ │
│ │  Promtail   │─┼──────────┼─│  Promtail   │─┼──────────┼─│  Promtail   │ │
│ │  (Logs)     │ │ Forward  │ │  (Logs)     │ │ Forward  │ │  (Logs)     │ │
│ └─────────────┘ │          │ └─────────────┘ │          │ └─────────────┘ │
└─────────────────┘          └─────────────────┘          └─────────────────┘
```

### Fleet Health Dashboard Metrics

```yaml
# Grafana dashboard for fleet overview
# Key metrics to monitor per cluster:

# Cluster Health
- name: "Cluster API Availability"
  expr: up{job="kubernetes-apiservers"}

- name: "Node Ready Status"
  expr: sum(kube_node_status_condition{condition="Ready",status="true"}) by (cluster)

- name: "Pod Health Ratio"
  expr: |
    sum(kube_pod_status_phase{phase="Running"}) by (cluster) /
    sum(kube_pod_status_phase) by (cluster)

# Fleet Sync Status
- name: "GitOps Sync Status"
  expr: argocd_app_info{sync_status="Synced"}

- name: "Last Successful Sync"
  expr: argocd_app_sync_total{result="Succeeded"}

# Security
- name: "Policy Violations"
  expr: sum(gatekeeper_violations) by (cluster, constraint)

- name: "Failed Auth Attempts"
  expr: sum(rate(apiserver_authentication_failures[5m])) by (cluster)

# Resource Utilization
- name: "Cluster CPU Usage"
  expr: |
    sum(rate(container_cpu_usage_seconds_total[5m])) by (cluster) /
    sum(machine_cpu_cores) by (cluster)

- name: "Cluster Memory Usage"
  expr: |
    sum(container_memory_working_set_bytes) by (cluster) /
    sum(machine_memory_bytes) by (cluster)
```

---

## Network Architecture

### Private Fleet Network

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                      PRIVATE FLEET NETWORK ARCHITECTURE                          │
└─────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────┐
│                           YOUR MANAGEMENT NETWORK                                │
│                              (Private VPC)                                       │
│                                                                                  │
│  ┌──────────────────────────────────────────────────────────────────────────┐   │
│  │                         HUB CLUSTER                                       │   │
│  │                       (10.100.0.0/16)                                     │   │
│  │                                                                           │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │   │
│  │  │  Rancher    │  │  ArgoCD     │  │  Vault      │  │  Thanos     │      │   │
│  │  │  10.100.1.x │  │  10.100.2.x │  │  10.100.3.x │  │  10.100.4.x │      │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘      │   │
│  │                                                                           │   │
│  │  ┌─────────────────────────────────────────────────────────────────────┐ │   │
│  │  │                      Tailscale Mesh Network                         │ │   │
│  │  │                         (100.x.x.x/8)                               │ │   │
│  │  └─────────────────────────────────────────────────────────────────────┘ │   │
│  └────────────────────────────────────┬─────────────────────────────────────┘   │
└───────────────────────────────────────┼─────────────────────────────────────────┘
                                        │
                                   Tailscale/
                                   WireGuard
                                        │
         ┌──────────────────────────────┼──────────────────────────────┐
         │                              │                              │
         ▼                              ▼                              ▼
┌─────────────────────┐     ┌─────────────────────┐     ┌─────────────────────┐
│   CLIENT A NETWORK  │     │   CLIENT B NETWORK  │     │   CLIENT C NETWORK  │
│    (DigitalOcean)   │     │       (AWS)         │     │       (GCP)         │
│                     │     │                     │     │                     │
│ VPC: 10.0.0.0/16    │     │ VPC: 10.1.0.0/16    │     │ VPC: 10.2.0.0/16    │
│                     │     │                     │     │                     │
│ ┌─────────────────┐ │     │ ┌─────────────────┐ │     │ ┌─────────────────┐ │
│ │ DOKS Cluster    │ │     │ │ EKS Cluster     │ │     │ │ GKE Cluster     │ │
│ │                 │ │     │ │                 │ │     │ │                 │ │
│ │ ┌─────────────┐ │ │     │ │ ┌─────────────┐ │ │     │ │ ┌─────────────┐ │ │
│ │ │ Tailscale   │ │ │     │ │ │ Tailscale   │ │ │     │ │ │ Tailscale   │ │ │
│ │ │ Agent       │ │ │     │ │ │ Agent       │ │ │     │ │ │ Agent       │ │ │
│ │ │ (Subnet     │ │ │     │ │ │ (Subnet     │ │ │     │ │ │ (Subnet     │ │ │
│ │ │  Router)    │ │ │     │ │ │  Router)    │ │ │     │ │ │  Router)    │ │ │
│ │ └─────────────┘ │ │     │ │ └─────────────┘ │ │     │ │ └─────────────┘ │ │
│ │                 │ │     │ │                 │ │     │ │                 │ │
│ │ ┌─────────────┐ │ │     │ │ ┌─────────────┐ │ │     │ │ ┌─────────────┐ │ │
│ │ │ Fleet Agent │ │ │     │ │ │ Fleet Agent │ │ │     │ │ │ Fleet Agent │ │ │
│ │ └─────────────┘ │ │     │ │ └─────────────┘ │ │     │ │ └─────────────┘ │ │
│ └─────────────────┘ │     │ └─────────────────┘ │     │ └─────────────────┘ │
│                     │     │                     │     │                     │
│ NO PUBLIC API       │     │ NO PUBLIC API       │     │ NO PUBLIC API       │
│ (Outbound only)     │     │ (Outbound only)     │     │ (Outbound only)     │
└─────────────────────┘     └─────────────────────┘     └─────────────────────┘
```

### Connection Flow (Agent-Based)

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                    AGENT-BASED CONNECTION MODEL                                  │
│                    (No inbound ports required)                                   │
└─────────────────────────────────────────────────────────────────────────────────┘

┌───────────────────────────────────────────────────────────────────────────────┐
│  1. CLUSTER REGISTRATION                                                       │
│                                                                                │
│    Spoke Cluster                            Hub Cluster                        │
│    ┌─────────────┐                          ┌─────────────┐                   │
│    │ Fleet Agent │ ──── Register ─────────► │ Rancher/    │                   │
│    │             │ ◄─── Token + Config ──── │ Fleet Mgr   │                   │
│    └─────────────┘      (HTTPS 443)         └─────────────┘                   │
│                                                                                │
└───────────────────────────────────────────────────────────────────────────────┘

┌───────────────────────────────────────────────────────────────────────────────┐
│  2. CONTINUOUS SYNC (Agent pulls, never pushed to)                             │
│                                                                                │
│    Spoke Cluster                            Hub Cluster                        │
│    ┌─────────────┐                          ┌─────────────┐                   │
│    │ Fleet Agent │ ──── Poll for updates ─► │ Fleet Mgr   │                   │
│    │             │ ◄─── GitOps manifests ── │             │                   │
│    │             │                          │             │                   │
│    │ ArgoCD      │ ──── Pull manifests ───► │ Git Repo    │                   │
│    │ Agent       │                          │             │                   │
│    └─────────────┘                          └─────────────┘                   │
│                                                                                │
│    ALL CONNECTIONS ARE OUTBOUND FROM SPOKE                                     │
│                                                                                │
└───────────────────────────────────────────────────────────────────────────────┘

┌───────────────────────────────────────────────────────────────────────────────┐
│  3. MONITORING DATA FLOW                                                       │
│                                                                                │
│    Spoke Cluster                            Hub Cluster                        │
│    ┌─────────────┐                          ┌─────────────┐                   │
│    │ Prometheus  │ ──── Remote Write ─────► │ Thanos/     │                   │
│    │ + Thanos    │      (HTTPS 443)         │ Mimir       │                   │
│    │ Sidecar     │                          │             │                   │
│    │             │                          │             │                   │
│    │ Promtail    │ ──── Push Logs ────────► │ Loki        │                   │
│    │             │      (HTTPS 443)         │             │                   │
│    └─────────────┘                          └─────────────┘                   │
│                                                                                │
└───────────────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Guide

### Step 1: Set Up Hub Cluster

```bash
# Create hub cluster (using Civo as example - cheapest)
civo kubernetes create hub-cluster \
  --size g4s.kube.medium \
  --nodes 3 \
  --wait

# Install Rancher
helm repo add rancher-stable https://releases.rancher.com/server-charts/stable
helm install rancher rancher-stable/rancher \
  --namespace cattle-system \
  --create-namespace \
  --set hostname=rancher.yourdomain.com \
  --set bootstrapPassword=admin \
  --set ingress.tls.source=letsEncrypt \
  --set letsEncrypt.email=admin@yourdomain.com

# Install ArgoCD
kubectl create namespace argocd
kubectl apply -n argocd -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml

# Install Vault for secrets
helm repo add hashicorp https://helm.releases.hashicorp.com
helm install vault hashicorp/vault \
  --namespace vault \
  --create-namespace \
  --set server.ha.enabled=true \
  --set server.ha.replicas=3
```

### Step 2: Set Up Private Network (Tailscale)

```bash
# On hub cluster - install Tailscale operator
helm repo add tailscale https://pkgs.tailscale.com/helmcharts
helm install tailscale-operator tailscale/tailscale-operator \
  --namespace tailscale \
  --create-namespace \
  --set oauth.clientId="${TS_OAUTH_CLIENT_ID}" \
  --set oauth.clientSecret="${TS_OAUTH_CLIENT_SECRET}"

# Expose Rancher via Tailscale (private access only)
kubectl apply -f - <<EOF
apiVersion: v1
kind: Service
metadata:
  name: rancher-tailscale
  namespace: cattle-system
  annotations:
    tailscale.com/expose: "true"
    tailscale.com/hostname: "rancher"
spec:
  selector:
    app: rancher
  ports:
    - port: 443
      targetPort: 443
  type: ClusterIP
EOF
```

### Step 3: Register Spoke Clusters

```bash
# Generate cluster import command from Rancher UI or API
# Run on each spoke cluster:

# Option A: Rancher agent
curl --insecure -sfL https://rancher.yourdomain.com/v3/import/xxxxx.yaml | kubectl apply -f -

# Option B: Fleet agent (lightweight)
helm install fleet-agent fleet/fleet-agent \
  --namespace cattle-fleet-system \
  --create-namespace \
  --set apiServerURL=https://rancher.yourdomain.com \
  --set clusterLabels.env=production \
  --set clusterLabels.client=client-a \
  --set clusterLabels.region=us-east
```

### Step 4: Configure GitOps

```yaml
# fleet.yaml - Root configuration
apiVersion: fleet.cattle.io/v1alpha1
kind: GitRepo
metadata:
  name: fleet-infra
  namespace: fleet-default
spec:
  repo: https://github.com/yourorg/fleet-repo
  branch: main
  paths:
    - bundles/
  targets:
    - name: all-clusters
      clusterSelector: {}
---
# ArgoCD ApplicationSet for multi-cluster
apiVersion: argoproj.io/v1alpha1
kind: ApplicationSet
metadata:
  name: cluster-addons
  namespace: argocd
spec:
  generators:
    - clusters:
        selector:
          matchLabels:
            env: production
  template:
    metadata:
      name: '{{name}}-addons'
    spec:
      project: default
      source:
        repoURL: https://github.com/yourorg/fleet-repo
        targetRevision: HEAD
        path: 'clusters/{{metadata.labels.client}}/{{name}}'
      destination:
        server: '{{server}}'
        namespace: kube-system
      syncPolicy:
        automated:
          prune: true
          selfHeal: true
```

### Step 5: Deploy Monitoring Stack

```bash
# Install Thanos on hub cluster
helm repo add bitnami https://charts.bitnami.com/bitnami
helm install thanos bitnami/thanos \
  --namespace monitoring \
  --create-namespace \
  --set query.enabled=true \
  --set queryFrontend.enabled=true \
  --set compactor.enabled=true \
  --set storegateway.enabled=true \
  --set receive.enabled=true

# Install Prometheus + Thanos sidecar on spoke clusters (via Fleet)
# This is deployed via GitOps bundle
```

---

## Quick Reference

### Fleet Commands

```bash
# List all clusters
kubectl get clusters.fleet.cattle.io -A

# Check bundle status
kubectl get bundles -A

# Force sync a cluster
kubectl annotate cluster <cluster-name> -n fleet-default \
  fleet.cattle.io/force-sync=$(date +%s)

# View cluster labels
kubectl get clusters.fleet.cattle.io -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.metadata.labels}{"\n"}{end}'
```

### ArgoCD Multi-Cluster Commands

```bash
# List all clusters
argocd cluster list

# Add a cluster
argocd cluster add <context-name> --name <cluster-name>

# Sync all apps in a project
argocd app sync -l project=production

# Get app status across clusters
argocd app list --selector env=production
```

### Security Audit Commands

```bash
# Check policy violations
kubectl get constraints -A
kubectl get violations -A

# Audit RBAC
kubectl auth can-i --list --as=system:serviceaccount:tenant-a:default

# Check network policies
kubectl get networkpolicies -A
```

---

## Summary

| Requirement | Solution |
|-------------|----------|
| **Fleet Management** | Rancher + Fleet |
| **GitOps Deployments** | ArgoCD/Flux + Fleet bundles |
| **Private Network** | Tailscale mesh (no public APIs) |
| **Multi-Cluster Monitoring** | Thanos + Grafana |
| **Secrets Management** | HashiCorp Vault + ESO |
| **Policy Enforcement** | OPA Gatekeeper / Kyverno |
| **RBAC** | Per-tenant namespaces + roles |
| **Patch Deployment** | Fleet bundles with wave rollout |
